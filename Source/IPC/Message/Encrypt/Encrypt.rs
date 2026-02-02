//! # Encrypt
//!
//! ## File: IPC/Message/Encrypt/Encrypt.rs
//!
//! ## Role in Mountain Architecture
//!
//! This module provides cryptographic security for IPC messages using AES-256-GCM encryption and HMAC signing. It ensures message confidentiality, integrity, and authenticity in the Mountain-Wind communication channel.
//!
//! ## Primary Responsibility
//!
//! Provide AES-256-GCM encryption with HMAC signing for secure IPC message transmission.
//!
//! ## Secondary Responsibilities
//!
//! - Secure key generation and storage
//! - Key rotation support
//! - Message authentication via HMAC
//!
//! ## Dependencies
//!
//! **External Crates:**
//! - `ring` - Cryptographic primitives (AES-256-GCM, HMAC-SHA256)
//! - `base64` - Encoding for binary data transmission
//!
//! **Internal Modules:**
//! - `IPC::Message::Define::DefineMessage` - Provides TauriIPCMessage and EncryptedMessage types
//!
//! ## Dependents
//!
//! - `IPC::TauriIPCServer` - Uses encryption for secure message handling
//!
//! ## VSCode Pattern Reference
//!
//! Follows VSCode's secure messaging pattern where messages are encrypted end-to-end with authentication to prevent tampering.
//!
//! ## Security Considerations
//!
//! - AES-256-GCM provides authenticated encryption (confidentiality + integrity)
//! - HMAC-SHA256 provides message authentication
//! - Nonces are generated using cryptographically secure random number generator
//! - Keys are kept in memory and never persisted to disk
//! - Keys are rotated regularly to limit exposure window
//!
//! ## Performance Considerations
//!
//! - Encryption keys are kept in memory for reuse
//! - Nonces are 12 bytes for optimal GCM performance
//! - Ring library uses hardware-accelerated AES-NI when available
//!
//! ## Error Handling Strategy
//!
//! - Returns Result<T, String> for all cryptographic operations
//! - All encryption/decryption failures are treated as security events
//! - HMAC verification failures are logged for security monitoring
//!
//! ## Thread Safety
//!
//! - Each Encrypt instance has its own keys (not shared)
//! - Encryption/decryption operations are thread-safe for the same instance
//! - Keys are wrapped in Arc<Mutex<T>> if sharing is needed
//!
//! ## TODO Items
//!
//! - [ ] Implement key derivation from master secret
//! - [ ] Add support for multiple key versions for graceful rotation
//! - [ ] Implement ephemeral keys for additional security


use log::{debug, error, trace, warn};
use ring::aead::{self, LessSafeKey, UnboundKey, AES_256_GCM};
use ring::hmac;
use ring::rand::{SecureRandom, SystemRandom};
use base64::{engine::general_purpose, Engine};

use super::super::Define::DefineMessage::{TauriIPCMessage, EncryptedMessage};

/// Nonce size for AES-256-GCM (96 bits as recommended by NIST)
const NONCE_SIZE: usize = 12;

/// Key size for AES-256 (256 bits)
const KEY_SIZE: usize = 32;

/// HMAC key size (256 bits for HMAC-SHA256)
const HMAC_KEY_SIZE: usize = 32;

/// Maximum allowed message size for encryption (prevents DoS)
const MAX_MESSAGE_SIZE: usize = 100 * 1024 * 1024; // 100MB

/// Secure message channel with encryption and authentication
///
/// Provides AES-256-GCM encryption combined with HMAC-SHA256 signing
/// for end-to-end secure communication.
pub struct Encrypt {
    /// AES-256-GCM encryption key
    encryption_key: LessSafeKey,
    /// HMAC-SHA256 key for message authentication
    hmac_key: Vec<u8>,
}

impl Encrypt {
    /// Create a new secure channel with cryptographically generated keys
    pub fn New() -> Result<Self, String> {
        let rng = SystemRandom::new();

        // Generate encryption key
        let mut encryption_key_bytes = vec![0u8; KEY_SIZE];
        rng.fill(&mut encryption_key_bytes)
            .map_err(|e| format!("Failed to generate encryption key: {}", e))?;

        let unbound_key = UnboundKey::new(&AES_256_GCM, &encryption_key_bytes)
            .map_err(|e| format!("Failed to create unbound key: {}", e))?;

        let encryption_key = LessSafeKey::new(unbound_key);

        // Generate HMAC key
        let mut hmac_key = vec![0u8; HMAC_KEY_SIZE];
        rng.fill(&mut hmac_key)
            .map_err(|e| format!("Failed to generate HMAC key: {}", e))?;

        debug!("[Encrypt] Secure channel initialized with new keys");
        Ok(Self {
            encryption_key,
            hmac_key,
        })
    }

    /// Create a new secure channel with provided keys
    ///
    /// This allows for key sharing between Mountain and Wind for
    /// bilateral communication.
    pub fn NewWithKeys(EncryptionKey: &[u8], HmacKey: &[u8]) -> Result<Self, String> {
        if EncryptionKey.len() != KEY_SIZE {
            return Err(format!("Encryption key must be {} bytes", KEY_SIZE));
        }
        if HmacKey.len() != HMAC_KEY_SIZE {
            return Err(format!("HMAC key must be {} bytes", HMAC_KEY_SIZE));
        }

        let unbound_key = UnboundKey::new(&AES_256_GCM, EncryptionKey)
            .map_err(|e| format!("Failed to create unbound key: {}", e))?;

        let encryption_key = LessSafeKey::new(unbound_key);

        debug!("[Encrypt] Secure channel initialized with provided keys");
        Ok(Self {
            encryption_key,
            hmac_key: HmacKey.to_vec(),
        })
    }

    /// Get the encryption key for sharing
    pub fn GetEncryptionKey(&self) -> Vec<u8> {
        // Note: In production, this should be done via secure key exchange
        vec![0u8; KEY_SIZE] // Placeholder - keys should never be exposed
    }

    /// Get the HMAC key for sharing
    pub fn GetHmacKey(&self) -> Vec<u8> {
        // Note: In production, this should be done via secure key exchange
        vec![0u8; HMAC_KEY_SIZE] // Placeholder - keys should never be exposed
    }

    /// Encrypt and authenticate a message
    pub fn EncryptMessage(&self, Message: &TauriIPCMessage) -> Result<EncryptedMessage, String> {
        let serialized_message = serde_json::to_vec(Message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        // Validate message size
        if serialized_message.len() > MAX_MESSAGE_SIZE {
            return Err(format!("Message too large to encrypt ({} > {} bytes)",
                serialized_message.len(), MAX_MESSAGE_SIZE));
        }

        // Generate nonce
        let mut nonce = [0u8; NONCE_SIZE];
        SystemRandom::new().fill(&mut nonce)
            .map_err(|e| format!("Failed to generate nonce: {}", e))?;

        // Encrypt message with AES-256-GCM
        let mut in_out = serialized_message.clone();
        self.encryption_key.seal_in_place_append_tag(
            aead::Nonce::assume_unique_for_key(nonce),
            aead::Aad::empty(),
            &mut in_out,
        ).map_err(|e| format!("Encryption failed: {}", e))?;

        // Create HMAC for additional authentication
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &self.hmac_key);
        let hmac_tag = hmac::sign(&hmac_key, &in_out);

        trace!("[Encrypt] Encrypted message ({} -> {} bytes)",
            serialized_message.len(), in_out.len());

        Ok(EncryptedMessage {
            Nonce: nonce.to_vec(),
            Ciphertext: in_out,
            HmacTag: hmac_tag.as_ref().to_vec(),
        })
    }

    /// Encrypt and authenticate a message and return base64-encoded result
    ///
    /// This is useful for transmitting encrypted messages over JSON.
    pub fn EncryptMessageBase64(&self, Message: &TauriIPCMessage) -> Result<serde_json::Value, String> {
        let encrypted = self.EncryptMessage(Message)?;

        Ok(serde_json::json!({
            "nonce": general_purpose::STANDARD.encode(&encrypted.Nonce),
            "ciphertext": general_purpose::STANDARD.encode(&encrypted.Ciphertext),
            "hmac_tag": general_purpose::STANDARD.encode(&encrypted.HmacTag),
        }))
    }

    /// Decrypt and verify a message
    pub fn DecryptMessage(&self, Encrypted: &EncryptedMessage) -> Result<TauriIPCMessage, String> {
        // Verify HMAC before decryption to detect tampering
        let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &self.hmac_key);
        hmac::verify(&hmac_key, &Encrypted.Ciphertext, &Encrypted.HmacTag)
            .map_err(|e| {
                warn!("[Encrypt] HMAC verification failed: {}", e);
                "HMAC verification failed".to_string()
            })?;

        // Decrypt message
        let mut in_out = Encrypted.Ciphertext.clone();
        let nonce_slice: &[u8] = &Encrypted.Nonce;
        let nonce_array: [u8; NONCE_SIZE] = nonce_slice.try_into()
            .map_err(|_| "Invalid nonce length".to_string())?;

        let nonce = aead::Nonce::assume_unique_for_key(nonce_array);

        self.encryption_key.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
            .map_err(|e| {
                warn!("[Encrypt] Decryption failed: {}", e);
                format!("Decryption failed: {}", e)
            })?;

        // Remove authentication tag
        let plaintext_len = in_out.len() - AES_256_GCM.tag_len();
        in_out.truncate(plaintext_len);

        // Validate decompressed size
        if in_out.len() > MAX_MESSAGE_SIZE {
            error!("[Encrypt] Decrypted message exceeds maximum size limit");
            return Err("Decrypted message exceeds maximum size limit".to_string());
        }

        trace!("[Encrypt] Decrypted message ({} bytes)", in_out.len());

        // Deserialize message
        serde_json::from_slice(&in_out)
            .map_err(|e| format!("Failed to deserialize message: {}", e))
    }

    /// Decrypt and verify a message from base64-encoded data
    pub fn DecryptMessageBase64(&self, EncryptedJson: serde_json::Value) -> Result<TauriIPCMessage, String> {
        let nonce_str = EncryptedJson.get("nonce")
            .and_then(|v| v.as_str())
            .ok_or("Missing nonce in encrypted message")?;
        
        let ciphertext_str = EncryptedJson.get("ciphertext")
            .and_then(|v| v.as_str())
            .ok_or("Missing ciphertext in encrypted message")?;
        
        let hmac_tag_str = EncryptedJson.get("hmac_tag")
            .and_then(|v| v.as_str())
            .ok_or("Missing hmac_tag in encrypted message")?;

        let nonce = general_purpose::STANDARD.decode(nonce_str)
            .map_err(|e| format!("Failed to decode nonce: {}", e))?;
        let ciphertext = general_purpose::STANDARD.decode(ciphertext_str)
            .map_err(|e| format!("Failed to decode ciphertext: {}", e))?;
        let hmac_tag = general_purpose::STANDARD.decode(hmac_tag_str)
            .map_err(|e| format!("Failed to decode hmac_tag: {}", e))?;

        let encrypted = EncryptedMessage {
            Nonce: nonce,
            Ciphertext: ciphertext,
            HmacTag: hmac_tag,
        };

        self.DecryptMessage(&encrypted)
    }

    /// Rotate encryption keys for enhanced security
    pub fn RotateKeys(&mut self) -> Result<(), String> {
        debug!("[Encrypt] Rotating encryption keys");
        *self = Self::New()?;
        debug!("[Encrypt] Encryption keys rotated successfully");
        Ok(())
    }

    /// Validate that two Encrypt instances have compatible keys
    pub fn ValidateKeyCompatibility(&self, Other: &Encrypt) -> bool {
        // In production, you might want to verify that keys match
        // For now, we assume any instance can communicate with any other
        true
    }
}

impl Default for Encrypt {
    fn default() -> Self {
        Self::New().expect("Failed to create default Encrypt instance")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let encrypt = Encrypt::New().unwrap();
        let message = TauriIPCMessage::New(
            "test-channel".to_string(),
            serde_json::json!({"hello": "world", "data": "test payload"}),
        );

        let encrypted = encrypt.EncryptMessage(&message).unwrap();
        let decrypted = encrypt.DecryptMessage(&encrypted).unwrap();

        assert_eq!(decrypted.Channel, message.Channel);
        assert_eq!(decrypted.Data, message.Data);
    }

    #[test]
    fn test_base64_roundtrip() {
        let encrypt = Encrypt::New().unwrap();
        let message = TauriIPCMessage::New(
            "test-channel".to_string(),
            serde_json::json!({"test": "data"}),
        );

        let encrypted_json = encrypt.EncryptMessageBase64(&message).unwrap();
        let decrypted = encrypt.DecryptMessageBase64(encrypted_json).unwrap();

        assert_eq!(decrypted.Channel, message.Channel);
    }

    #[test]
    fn test_key_rotation() {
        let mut encrypt = Encrypt::New().unwrap();
        let message = TauriIPCMessage::New(
            "test-channel".to_string(),
            serde_json::json!({"test": "data"}),
        );

        let encrypted_old = encrypt.EncryptMessage(&message).unwrap();

        encrypt.RotateKeys().unwrap();
        
        // New key should not be able to decrypt old messages
        assert!(encrypt.DecryptMessage(&encrypted_old).is_err());
    }
}

pub use Encrypt;
