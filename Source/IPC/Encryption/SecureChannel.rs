//! # Secure Message Channel (IPC Encryption)
//!
//! ## RESPONSIBILITIES
//! This module provides secure message channels using AES-256-GCM encryption
//! with HMAC authentication. It ensures message confidentiality and integrity
//! for sensitive IPC communications.
//!
//! ## ARCHITECTURAL ROLE
//! This module is part of the security layer in the IPC architecture, providing
//! end-to-end encryption for sensitive messages.
//!
//! ## KEY COMPONENTS
//!
//! - **SecureMessageChannel**: Encryption channel with AES-256-GCM + HMAC
//! - **EncryptedMessage**: Encrypted message structure with nonce and HMAC tag
//!
//! ## ERROR HANDLING
//! All encryption/decryption operations return Result types with descriptive
//! error messages for failures.
//!
//! ## LOGGING
// Debug-level logging for key operations, error for failures.
//
// ## Performance Considerations
// - AES-256-GCM provides hardware-accelerated encryption on modern CPUs
// - Nonce-based encryption ensures unique ciphertexts
// - HMAC provides message authentication and integrity verification
//
// ## TODO
// - Add encryption key rotation
// - Implement symmetric key exchange protocol
// - Add support for multiple encryption algorithms
// - Implement message replay attack prevention

use ring::{
	aead::{self, AES_256_GCM, LessSafeKey, UnboundKey},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use log::debug;

use super::super::Message::TauriIPCMessage;

/// Encrypted message structure
///
/// This structure contains the encrypted message data along with the nonce
/// and HMAC tag needed for decryption and verification.
///
/// ## Message Structure
///
/// ```text
/// EncryptedMessage {
///     nonce: [u8; 12],      // Unique value for each encryption
///     ciphertext: Vec<u8>,  // Encrypted message + auth tag
///     hmac_tag: Vec<u8>,    // HMAC for message authentication
/// }
/// ```
///
/// ## Example Usage
///
/// ```rust,ignore
/// let encrypted = EncryptedMessage {
///     nonce: vec![1, 2, 3, ...],
///     ciphertext: vec![...],
///     hmac_tag: vec![...],
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
	/// Nonce used for encryption (12 bytes for AES-256-GCM)
	pub nonce: Vec<u8>,

	/// Encrypted message data with authentication tag
	pub ciphertext: Vec<u8>,

	/// HMAC tag for message authentication
	pub hmac_tag: Vec<u8>,
}

impl EncryptedMessage {
	/// Create a new encrypted message
	pub fn new(nonce: Vec<u8>, ciphertext: Vec<u8>, hmac_tag: Vec<u8>) -> Self {
		Self {
			nonce,
			ciphertext,
			hmac_tag,
		}
	}

	/// Validate the message structure
	pub fn is_valid(&self) -> bool {
		self.nonce.len() == 12 // AES-256-GCM requires 12-byte nonce
			&& !self.ciphertext.is_empty()
			&& !self.hmac_tag.is_empty()
	}
}

/// Secure message channel with encryption and authentication
///
/// This structure provides AES-256-GCM encryption with HMAC authentication
/// for secure IPC communication. It ensures message confidentiality and integrity.
///
/// ## Encryption Flow
///
/// ```text
/// TauriIPCMessage
///     |
///     | 1. Serialize to JSON
///     v
/// Serialized bytes
///     |
///     | 2. Encrypt with AES-256-GCM
///     v
/// Encrypted bytes + auth tag
///     |
///     | 3. Generate HMAC
///     v
/// EncryptedMessage (nonce, ciphertext, hmac_tag)
/// ```
///
/// ## Decryption Flow
///
/// ```text
/// EncryptedMessage
///     |
///     | 1. Verify HMAC
///     v
/// HMAC valid
///     |
///     | 2. Decrypt with AES-256-GCM
///     v
/// Serialized bytes
///     |
///     | 3. Deserialize to TauriIPCMessage
///     v
/// TauriIPCMessage
/// ```
///
/// ## Security Features
///
/// - **AES-256-GCM**: Industry-standard authenticated encryption
/// - **Unique Nonces**: Each encryption uses a unique nonce
/// - **HMAC Authentication**: Additional layer of message authentication
/// - **Secure Random Generation**: Cryptographically secure random keys
///
/// ## Example Usage
///
/// ```rust,ignore
/// let secure_channel = SecureMessageChannel::new()?;
///
/// // Encrypt a message
/// let encrypted = secure_channel.encrypt_message(&message)?;
///
/// // Decrypt a message
/// let decrypted = secure_channel.decrypt_message(&encrypted)?;
///
/// // Rotate keys
/// secure_channel.rotate_keys()?;
/// ```
pub struct SecureMessageChannel {
	/// AES-256-GCM encryption key
	encryption_key: LessSafeKey,

	/// HMAC key for message authentication
	hmac_key: Vec<u8>,
}

impl SecureMessageChannel {
	/// Create a new secure channel with randomly generated keys
	///
	/// This method generates cryptographically secure random keys for
	/// encryption and HMAC authentication.
	///
	/// ## Returns
	/// - `Ok(SecureMessageChannel)`: New secure channel
	/// - `Err(String)`: Error message if key generation fails
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let secure_channel = SecureMessageChannel::new()?;
	/// ```
	pub fn new() -> Result<Self, String> {
		debug!("[SecureMessageChannel] Creating new secure channel");

		let rng = SystemRandom::new();

		// Generate 256-bit (32-byte) encryption key
		let mut encryption_key_bytes = vec![0u8; 32];
		rng.fill(&mut encryption_key_bytes)
			.map_err(|e| format!("Failed to generate encryption key: {}", e))?;

		let unbound_key = UnboundKey::new(&AES_256_GCM, &encryption_key_bytes)
			.map_err(|e| format!("Failed to create unbound key: {}", e))?;

		let encryption_key = LessSafeKey::new(unbound_key);

		// Generate 256-bit HMAC key
		let mut hmac_key = vec![0u8; 32];
		rng.fill(&mut hmac_key)
			.map_err(|e| format!("Failed to generate HMAC key: {}", e))?;

		debug!("[SecureMessageChannel] Secure channel created successfully");

		Ok(Self {
			encryption_key,
			hmac_key,
		})
	}

	/// Encrypt and authenticate a message
	///
	/// This method serializes the message, encrypts it with AES-256-GCM,
	/// and adds an HMAC tag for authentication.
	///
	/// ## Parameters
	/// - `message`: The message to encrypt
	///
	/// ## Returns
	/// - `Ok(EncryptedMessage)`: Encrypted message with nonce and HMAC tag
	/// - `Err(String)`: Error message if encryption fails
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let encrypted = secure_channel.encrypt_message(&message)?;
	/// ```
	pub fn encrypt_message(&self, message: &TauriIPCMessage) -> Result<EncryptedMessage, String> {
		debug!("[SecureMessageChannel] Encrypting message on channel: {}", message.channel);

		// Serialize message to bytes
		let serialized_message =
			serde_json::to_vec(message).map_err(|e| format!("Failed to serialize message: {}", e))?;

		// Generate unique 12-byte nonce (required for AES-256-GCM)
		let mut nonce = [0u8; 12];
		SystemRandom::new()
			.fill(&mut nonce)
			.map_err(|e| format!("Failed to generate nonce: {}", e))?;

		// Encrypt with AES-256-GCM (authenticated encryption)
		let mut in_out = serialized_message.clone();
		self.encryption_key
			.seal_in_place_append_tag(
				aead::Nonce::assume_unique_for_key(nonce),
				aead::Aad::empty(),
				&mut in_out,
			)
			.map_err(|e| format!("Encryption failed: {}", e))?;

		// Generate HMAC for additional authentication
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &self.hmac_key);
		let hmac_tag = hmac::sign(&hmac_key, &in_out);

		let encrypted_message = EncryptedMessage {
			nonce: nonce.to_vec(),
			ciphertext: in_out,
			hmac_tag: hmac_tag.as_ref().to_vec(),
		};

		debug!(
			"[SecureMessageChannel] Message encrypted: {} bytes -> {} bytes",
			serialized_message.len(),
			encrypted_message.ciphertext.len()
		);

		Ok(encrypted_message)
	}

	/// Decrypt and verify a message
	///
	/// This method verifies the HMAC tag, decrypts the message with
	/// AES-256-GCM, and deserializes it back to the original format.
	///
	/// ## Parameters
	/// - `encrypted`: The encrypted message to decrypt
	///
	/// ## Returns
	/// - `Ok(TauriIPCMessage)`: Decrypted message
	/// - `Err(String)`: Error message if decryption or verification fails
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let decrypted = secure_channel.decrypt_message(&encrypted)?;
	/// ```
	pub fn decrypt_message(&self, encrypted: &EncryptedMessage) -> Result<TauriIPCMessage, String> {
		debug!("[SecureMessageChannel] Decrypting message");

		// Verify HMAC first (detect tampering)
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &self.hmac_key);
		hmac::verify(&hmac_key, &encrypted.ciphertext, &encrypted.hmac_tag)
			.map_err(|_| "HMAC verification failed - message may be tampered".to_string())?;

		// Convert nonce slice to array
		let nonce_slice: &[u8] = &encrypted.nonce;
		let nonce_array: [u8; 12] = nonce_slice
			.try_into()
			.map_err(|_| "Invalid nonce length - must be 12 bytes".to_string())?;

		let nonce = aead::Nonce::assume_unique_for_key(nonce_array);

		// Decrypt with AES-256-GCM
		let mut in_out = encrypted.ciphertext.clone();
		self.encryption_key
			.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
			.map_err(|e| format!("Decryption failed: {}", e))?;

		// Remove the authentication tag (last 16 bytes for AES-256-GCM)
		let plaintext_len = in_out.len() - AES_256_GCM.tag_len();
		in_out.truncate(plaintext_len);

		// Deserialize message
		let message: TauriIPCMessage =
			serde_json::from_slice(&in_out).map_err(|e| format!("Failed to deserialize message: {}", e))?;

		debug!(
			"[SecureMessageChannel] Message decrypted successfully on channel: {}",
			message.channel
		);

		Ok(message)
	}

	/// Rotate encryption keys
	///
	/// This method generates new encryption and HMAC keys, effectively
	/// rotating the security credentials for the channel.
	///
	/// ## Returns
	/// - `Ok(())`: Keys rotated successfully
	/// - `Err(String)`: Error message if key rotation fails
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// secure_channel.rotate_keys()?;
	/// ```
	pub fn rotate_keys(&mut self) -> Result<(), String> {
		debug!("[SecureMessageChannel] Rotating encryption keys");

		*self = Self::new()?;

		debug!("[SecureMessageChannel] Keys rotated successfully");

		Ok(())
	}

	/// Get the HMAC tag length (in bytes)
	pub fn hmac_tag_length(&self) -> usize {
		32 // HMAC-SHA256 produces 32-byte tags
	}

	/// Get the nonce length (in bytes)
	pub fn nonce_length(&self) -> usize {
		12 // AES-256-GCM requires 12-byte nonces
	}

	/// Get the authentication tag length (in bytes)
	pub fn auth_tag_length(&self) -> usize {
		AES_256_GCM.tag_len()
	}

	/// Get the key length (in bytes)
	pub fn key_length(&self) -> usize {
		32 // AES-256 uses 32-byte keys
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::Element::Mountain::Source::IPC::Message::TauriIPCMessage;

	fn create_test_message() -> TauriIPCMessage {
		TauriIPCMessage::new(
			"test_channel".to_string(),
			serde_json::json!({
				"data": "sensitive information that should be encrypted",
				"id": 12345
			}),
			Some("test_sender".to_string()),
		)
	}

#[test]
	fn test_secure_channel_creation() {
		let channel = SecureMessageChannel::new();
		assert!(channel.is_ok());
	}

#[test]
	fn test_encrypt_and_decrypt() {
		let channel = SecureMessageChannel::new().unwrap();
		let original_message = create_test_message();

		// Encrypt
		let encrypted = channel.encrypt_message(&original_message).unwrap();
		assert!(encrypted.is_valid());

		// Decrypt
		let decrypted = channel.decrypt_message(&encrypted).unwrap();

		// Verify content
		assert_eq!(decrypted.channel, original_message.channel);
		assert_eq!(decrypted.data, original_message.data);
		assert_eq!(decrypted.sender, original_message.sender);
	}

#[test]
	fn test_encryption_produces_different_outputs() {
		let channel = SecureMessageChannel::new().unwrap();
		let message = create_test_message();

		let encrypted1 = channel.encrypt_message(&message).unwrap();
		let encrypted2 = channel.encrypt_message(&message).unwrap();

		// Each encryption should produce different output (due to unique nonces)
		assert_ne!(encrypted1.nonce, encrypted2.nonce);
		assert_ne!(encrypted1.ciphertext, encrypted2.ciphertext);
	}

#[test]
	fn test_tampered_message_fails_hmac_verification() {
		let channel = SecureMessageChannel::new().unwrap();
		let message = create_test_message();

		let mut encrypted = channel.encrypt_message(&message).unwrap();

		// Tamper with the ciphertext
		if !encrypted.ciphertext.is_empty() {
			encrypted.ciphertext[0] ^= 0xFF;
		}

		// Should fail HMAC verification
		let result = channel.decrypt_message(&encrypted);
		assert!(result.is_err());
		assert!(result.unwrap_err().contains("HMAC verification failed"));
	}

#[test]
	fn test_invalid_nonce_length() {
		let channel = SecureMessageChannel::new().unwrap();
		let message = create_test_message();

		let mut encrypted = channel.encrypt_message(&message).unwrap();

		// Corrupt the nonce length
		encrypted.nonce = vec![0u8; 16]; // Wrong length

		let result = channel.decrypt_message(&encrypted);
		assert!(result.is_err());
		assert!(result.unwrap_err().contains("Invalid nonce length"));
	}

#[test]
	fn test_message_channel_key_lengths() {
		let channel = SecureMessageChannel::new().unwrap();

		assert_eq!(channel.key_length(), 32);
		assert_eq!(channel.nonce_length(), 12);
		assert_eq!(channel.auth_tag_length(), 16); // AES-256-GCM
		assert_eq!(channel.hmac_tag_length(), 32); // HMAC-SHA256
	}

#[test]
	fn test_key_rotation() {
		let mut channel = SecureMessageChannel::new().unwrap();
		let message = create_test_message();

		// Encrypt with original keys
		let encrypted1 = channel.encrypt_message(&message).unwrap();

		// Rotate keys
		let result = channel.rotate_keys();
		assert!(result.is_ok());

		// Old encrypted message should still decode successfully
		let decrypted1 = channel.decrypt_message(&encrypted1).unwrap();
		assert_eq!(decrypted1.channel, message.channel);

		// New encryption should work with new keys
		let encrypted2 = channel.encrypt_message(&message).unwrap();
		let decrypted2 = channel.decrypt_message(&encrypted2).unwrap();
		assert_eq!(decrypted2.channel, message.channel);

		// Encrypted versions should be different
		assert_ne!(encrypted1.nonce, encrypted2.nonce);
	}

#[test]
	fn test_empty_message() {
		let channel = SecureMessageChannel::new().unwrap();
		let message = TauriIPCMessage::new(
			"test".to_string(),
			serde_json::json!(null),
			None,
		);

		let encrypted = channel.encrypt_message(&message).unwrap();
		let decrypted = channel.decrypt_message(&encrypted).unwrap();

		assert_eq!(decrypted.channel, "test");
	}
}
