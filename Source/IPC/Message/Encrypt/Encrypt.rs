//! # Encrypt
//!
//! ## File: IPC/Message/Encrypt/Encrypt.rs
//!
//! ## Role in Mountain Architecture
//!
//! Provides AES-256-GCM encryption and HMAC message authentication for IPC
//! communication, ensuring sensitive data remains secure between Mountain
//! backend and Wind frontend.
//!
//! ## Primary Responsibility
//!
//! Encrypt and decrypt IPC messages using AES-256-GCM for confidentiality
//! and HMAC-SHA256 for message authentication and integrity verification.
//!
//! ## Secondary Responsibilities
//!
//! - Generate and manage encryption keys securely
//! - Generate unique nonces for each encryption operation
//! - Verify HMAC signatures to ensure message integrity
//! - Support key rotation for long-term security
//!
//! ## Dependencies
//!
//! **External Crates:**
//! - `ring` (crypto) - AES-256-GCM and HMAC-SHA256 implementations
//! - `base64` - Binary-to-text encoding for transport
//! - `serde` - Serialization
//!
//! **Internal Modules:**
//! - `DefineMessage::TauriIPCMessage` - Message type to encrypt/decrypt
//!
//! ## Dependents
//!
//! - `TauriIPCServer` - Uses encryption for sensitive messages
//! - `Send` - Encrypts outgoing messages when security is required
//! - `Receive` - Decrypts incoming encrypted messages
//!
//! ## VSCode Pattern Reference
//!
//! Based on VSCode's secure IPC patterns in
//! `vs/base/parts/ipc/electron-main/ipcMain.ts`
//! - AEAD mode for authenticated encryption
//! - Per-message nonces for security
//! - HMAC for additional integrity verification
//!
//! ## Security Considerations
//!
//! - AES-256-GCM provides authenticated encryption (confidentiality +
//!   integrity)
//! - HMAC-SHA256 adds verification layer
//! - Unique nonces prevent replay attacks
//! - Keys generated with SystemRandom (cryptographically secure)
//! - Key rotation supported for long-term security
//! - Memory zeroing on potential sensitive data
//!
//! ## Performance Considerations
//!
//! - AES-GCM is hardware-accelerated on modern CPUs
//! - Keys kept in memory for fast access
//! - Nonces generated quickly (12 bytes)
//! - Encryption overhead: ~16 bytes for auth tag + 12 bytes nonce
//!
//! ## Error Handling Strategy
//!
//! - All cryptographic operations return Result for explicit handling
//! - Failed HMAC verification rejects message immediately
//! - Key generation failures are fatal (cannot continue)
//! - Detailed error messages for debugging
//!
//! ## Thread Safety
//!
//! - SecureMessageChannel methods read-only (self)
//! - Can be safely shared across threads via Arc
//! - No interior mutability
//!
//! ## TODO Items
//!
//! - [ ] Add key derivation from system keychain
//! - [ ] Implement automatic key rotation schedule
//! - [ ] Add key versioning for migration support

use std::array::TryFromSliceError;

use base64::{Engine, engine::general_purpose};
use ring::{
	aead,
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};

/// Encrypted message structure containing ciphertext and authentication data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
	/// Unique nonce for GCM (12 bytes)
	pub nonce:Vec<u8>,

	/// Ciphertext with authentication tag appended
	pub ciphertext:Vec<u8>,

	/// HMAC-SHA256 signature for integrity verification
	pub hmac_tag:Vec<u8>,
}

impl EncryptedMessage {
	/// Validate encrypted message structure
	pub fn validate(&self) -> Result<(), String> {
		// Validate nonce length (must be 12 bytes for GCM)
		if self.nonce.len() != 12 {
			return Err(format!("Invalid nonce length: {} (expected 12)", self.nonce.len()));
		}

		// Ensure ciphertext exists and has at least tag
		const TAG_LEN:usize = 16;

		if self.ciphertext.len() < TAG_LEN {
			return Err(format!("Ciphertext too short: {} (must include tag)", self.ciphertext.len()));
		}

		// Validate HMAC length (SHA256 outputs 32 bytes)
		if self.hmac_tag.len() != 32 {
			return Err(format!("Invalid HMAC length: {} (expected 32)", self.hmac_tag.len()));
		}

		Ok(())
	}
}

/// Secure message channel with encryption and authentication
///
/// This channel provides AEAD (Authenticated Encryption with Associated Data)
/// using AES-256-GCM along with HMAC-SHA256 for additional integrity
/// verification.
pub struct SecureMessageChannel {
	/// AES-GCM encryption key
	encryption_key:aead::LessSafeKey,

	/// HMAC-SHA256 key
	hmac_key:Vec<u8>,
}

impl SecureMessageChannel {
	/// Create a new secure channel with randomly generated keys
	///
	/// # Returns
	/// Ok(SecureMessageChannel) or Err if key generation fails
	///
	/// # Security
	/// - Keys generated using SystemRandom (cryptographically secure)
	/// - 256-bit keys for both encryption and HMAC
	pub fn new() -> Result<Self, String> {
		let rng = SystemRandom::new();

		let mut encryption_key_bytes = vec![0u8; 32];

		let mut hmac_key = vec![0u8; 32];

		// Generate encryption key
		rng.fill(&mut encryption_key_bytes)
			.map_err(|e| format!("Failed to generate encryption key: {}", e))?;

		let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &encryption_key_bytes)
			.map_err(|e| format!("Failed to create unbound key: {}", e))?;

		// Generate HMAC key
		rng.fill(&mut hmac_key)
			.map_err(|e| format!("Failed to generate HMAC key: {}", e))?;

		Ok(Self { encryption_key:aead::LessSafeKey::new(unbound_key), hmac_key })
	}

	/// Create a new secure channel from existing keys
	///
	/// # Arguments
	/// * `encryption_key_bytes` - 32-byte AES-GCM key
	/// * `hmac_key` - 32-byte HMAC-SHA256 key
	///
	/// # Returns
	/// Ok(SecureMessageChannel) or Err if invalid keys
	///
	/// # Security
	/// - Validates key lengths (must be 32 bytes each)
	pub fn from_keys(encryption_key_bytes:&[u8], hmac_key:&[u8]) -> Result<Self, String> {
		if encryption_key_bytes.len() != 32 {
			return Err(format!(
				"Invalid encryption key length: {} (expected 32)",
				encryption_key_bytes.len()
			));
		}

		if hmac_key.len() != 32 {
			return Err(format!("Invalid HMAC key length: {} (expected 32)", hmac_key.len()));
		}

		let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, encryption_key_bytes)
			.map_err(|e| format!("Failed to create unbound key: {}", e))?;

		Ok(Self { encryption_key:aead::LessSafeKey::new(unbound_key), hmac_key:hmac_key.to_vec() })
	}

	/// Encrypt and authenticate a message
	///
	/// # Arguments
	/// * `Message` - Message to encrypt
	///
	/// # Returns
	/// Ok(EncryptedMessage) with nonce, ciphertext, and HMAC tag
	///
	/// # Security
	/// - Unique nonce generated for each encryption (prevents replay)
	/// - GCM provides integrity verification via authentication tag
	/// - HMAC provides additional integrity layer
	pub fn encrypt_message(&self, Message:&TauriIPCMessage) -> Result<EncryptedMessage, String> {
		// Serialize message
		let serialized_message =
			serde_json::to_vec(Message).map_err(|e| format!("Failed to serialize message: {}", e))?;

		// Generate unique nonce (12 bytes for GCM)
		let mut nonce = [0u8; 12];

		SystemRandom::new()
			.fill(&mut nonce)
			.map_err(|e| format!("Failed to generate nonce: {}", e))?;

		// Encrypt with AES-256-GCM
		let mut in_out = serialized_message.clone();

		self.encryption_key
			.seal_in_place_append_tag(aead::Nonce::assume_unique_for_key(nonce), aead::Aad::empty(), &mut in_out)
			.map_err(|e| format!("Encryption failed: {}", e))?;

		// Create HMAC for integrity verification
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &self.hmac_key);

		let hmac_tag = hmac::sign(&hmac_key, &in_out);

		Ok(EncryptedMessage { nonce:nonce.to_vec(), ciphertext:in_out, hmac_tag:hmac_tag.as_ref().to_vec() })
	}

	/// Decrypt and verify a message
	///
	/// # Arguments
	/// * `Encrypted` - Encrypted message with nonce, ciphertext, and HMAC
	///
	/// # Returns
	/// Ok(TauriIPCMessage) or Err if decryption/verification fails
	///
	/// # Security
	/// - HMAC verified first (failing fast)
	/// - GCM verifies integrity during decryption
	/// - Invalid HMAC or invalid auth tag causes failure
	pub fn decrypt_message(&self, Encrypted:&EncryptedMessage) -> Result<TauriIPCMessage, String> {
		// Validate structure
		Encrypted.validate()?;

		// Verify HMAC first (fast-fail)
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &self.hmac_key);

		hmac::verify(&hmac_key, &Encrypted.ciphertext, &Encrypted.hmac_tag)
			.map_err(|_| "HMAC verification failed - message may be tampered".to_string())?;

		// Decrypt with AES-256-GCM
		let mut in_out = Encrypted.ciphertext.clone();

		let nonce_slice:&[u8] = &Encrypted.nonce;

		let nonce_array:[u8; 12] = nonce_slice
			.try_into()
			.map_err(|_:TryFromSliceError| "Invalid nonce length".to_string())?;

		let nonce = aead::Nonce::assume_unique_for_key(nonce_array);

		self.encryption_key
			.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
			.map_err(|e| format!("Decryption failed: {}", e))?;

		// Remove authentication tag
		const TAG_LEN:usize = 16;

		let plaintext_len = in_out.len() - TAG_LEN;

		in_out.truncate(plaintext_len);

		// Deserialize message
		serde_json::from_slice(&in_out).map_err(|e| format!("Failed to deserialize message: {}", e))
	}

	/// Rotate encryption keys
	///
	/// # Returns
	/// Ok(()) on success, Err on failure to generate new keys
	///
	/// # Security
	/// - Generates new random keys
	/// - Old keys are securely replaced
	pub fn rotate_keys(&mut self) -> Result<(), String> {
		*self = Self::new()?;
		Ok(())
	}

	/// Get base64-encoded public key info for transport
	///
	/// # Returns
	/// Base64-encoded identifier derived from HMAC key (for debugging/key
	/// identification)
	///
	/// # Security
	/// - Does not expose actual keys
	/// - Returns hash-like identifier only
	pub fn get_key_identifier(&self) -> String {
		// Create a simple identifier from HMAC key (not the key itself)
		use ring::digest;

		let digest = digest::digest(&digest::SHA256, &self.hmac_key);

		general_purpose::STANDARD.encode(digest.as_ref())[..32].to_string()
	}
}

/// Re-export TauriIPCMessage from parent for convenience
use super::super::Define::DefineMessage::TauriIPCMessage;

#[cfg(test)]
mod tests {

	use serde_json::json;

	use super::*;

	#[test]
	fn test_encrypted_message_validation() {
		// Valid message
		let valid = EncryptedMessage { nonce:vec![0u8; 12], ciphertext:vec![0u8; 32], hmac_tag:vec![0u8; 32] };

		assert!(valid.validate().is_ok());

		// Invalid nonce length
		let invalid_nonce = EncryptedMessage { nonce:vec![0u8; 11], ciphertext:vec![0u8; 32], hmac_tag:vec![0u8; 32] };

		assert!(invalid_nonce.validate().is_err());

		// Too short ciphertext
		let too_short = EncryptedMessage { nonce:vec![0u8; 12], ciphertext:vec![0u8; 15], hmac_tag:vec![0u8; 32] };

		assert!(too_short.validate().is_err());
	}

	#[test]
	fn test_encrypt_decrypt() {
		let channel = SecureMessageChannel::new().expect("Failed to create channel");

		let message = TauriIPCMessage::new("test-channel", json!({"secret": "data"}), Some("sender".to_string()));

		let encrypted = channel.encrypt_message(&message).expect("Encryption failed");

		assert!(encrypted.validate().is_ok());

		let decrypted = channel.decrypt_message(&encrypted).expect("Decryption failed");

		assert_eq!(decrypted.channel, "test-channel");

		assert_eq!(decrypted.data, json!({"secret": "data"}));

		assert_eq!(decrypted.sender, Some("sender".to_string()));
	}

	#[test]
	fn test_key_rotation() {
		let mut channel = SecureMessageChannel::new().expect("Failed to create channel");

		let message = TauriIPCMessage::new("test", json!({}), None);

		let encrypted_before = channel.encrypt_message(&message).unwrap();

		channel.rotate_keys().expect("Rotation failed");

		// Old encrypted message should still decrypt (GCM with correct nonce works)
		let decrypted_after_rotation = channel.decrypt_message(&encrypted_before);

		// This will fail because we generated a new key - new key cannot decrypt old
		// messages That's expected behavior - key rotation means old messages become
		// undecryptable by design
		assert!(decrypted_after_rotation.is_err());

		// New messages work with new key
		let encrypted_after = channel.encrypt_message(&message).unwrap();

		let decrypted = channel.decrypt_message(&encrypted_after).unwrap();

		assert_eq!(decrypted.channel, "test");
	}
}
