pub mod New;
pub mod IsValid;
pub mod New;
pub mod EncryptMessage;
pub mod DecryptMessage;
pub mod RotateKeys;
pub mod HmacTagLength;
pub mod NonceLength;
pub mod AuthTagLength;
pub mod KeyLength;

use ring::{
	aead::{self, AES_256_GCM, LessSafeKey, UnboundKey},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

/// Encrypted message structure
/// This structure contains the encrypted message data along with the nonce
/// and HMAC tag needed for decryption and verification.
/// ## Message Structure
/// ```text
/// EncryptedMessage {
///     nonce: [u8; 12],      // Unique value for each encryption
///     ciphertext: Vec<u8>,  // Encrypted message + auth tag
///     hmac_tag: Vec<u8>,    // HMAC for message authentication
/// }
/// ```
/// ## Example Usage
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
	pub nonce:Vec<u8>,

	/// Encrypted message data with authentication tag
	pub ciphertext:Vec<u8>,

	/// HMAC tag for message authentication
	pub hmac_tag:Vec<u8>,
}

/// Secure message channel with encryption and authentication
/// This structure provides AES-256-GCM encryption with HMAC authentication
/// for secure IPC communication. It ensures message confidentiality and
/// integrity.
/// ## Encryption Flow
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
/// ## Decryption Flow
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
/// ## Security Features
/// - **AES-256-GCM**: Industry-standard authenticated encryption
/// - **Unique Nonces**: Each encryption uses a unique nonce
/// - **HMAC Authentication**: Additional layer of message authentication
/// - **Secure Random Generation**: Cryptographically secure random keys
/// ## Example Usage
/// ```rust,ignore
/// let secure_channel = SecureMessageChannel::new()?;
/// // Encrypt a message
/// let encrypted = secure_channel.EncryptMessage(&message)?;
/// // Decrypt a message
/// let decrypted = secure_channel.DecryptMessage(&encrypted)?;
/// // Rotate keys
/// secure_channel.RotateKeys()?;
/// ```
pub struct SecureMessageChannel {
	/// AES-256-GCM encryption key
	encryption_key:LessSafeKey,

	/// HMAC key for message authentication
	hmac_key:Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Struct;
