//! # Secure Message Channel
//!
//! Advanced security enhancements for IPC messages including AES-256-GCM
//! encryption, HMAC authentication, and secure key management.

use std::{
	collections::HashMap,
	marker::PhantomData,
	sync::Arc,
	time::{Duration, SystemTime},
};

use log::{debug, error, info, trace, warn};
use ring::{
	aead::{self, AES_256_GCM, LessSafeKey, NONCE_LEN, UnboundKey},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use bincode::serde::{decode_from_slice, encode_to_vec};

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
	pub encryption_algorithm:String,
	pub key_rotation_interval_hours:u64,
	pub hmac_algorithm:String,
	pub nonce_size_bytes:usize,
	pub auth_tag_size_bytes:usize,
	pub max_message_size_bytes:usize,
}

impl Default for SecurityConfig {
	fn default() -> Self {
		Self {
			encryption_algorithm:"AES-256-GCM".to_string(),
			key_rotation_interval_hours:24, // Rotate keys daily
			hmac_algorithm:"HMAC-SHA256".to_string(),
			nonce_size_bytes:NONCE_LEN,
			auth_tag_size_bytes:AES_256_GCM.tag_len(),
			max_message_size_bytes:10 * 1024 * 1024, // 10MB
		}
	}
}

/// Encryption key with metadata
#[derive(Debug, Clone)]
struct EncryptionKey {
	key:LessSafeKey,
	created_at:SystemTime,
	key_id:String,
	usage_count:usize,
}

impl EncryptionKey {
	fn new(key_bytes:&[u8]) -> Result<Self, String> {
		let unbound_key =
			UnboundKey::new(&AES_256_GCM, key_bytes).map_err(|e| format!("Failed to create unbound key: {}", e))?;

		Ok(Self {
			key:LessSafeKey::new(unbound_key),
			created_at:SystemTime::now(),
			key_id:Self::generate_key_id(),
			usage_count:0,
		})
	}

	fn generate_key_id() -> String {
		let rng = SystemRandom::new();
		let mut id_bytes = [0u8; 8];
		rng.fill(&mut id_bytes).unwrap();
		hex::encode(id_bytes)
	}

	fn is_expired(&self, rotation_interval:Duration) -> bool {
		self.created_at.elapsed().unwrap_or_default() > rotation_interval
	}

	fn increment_usage(&mut self) { self.usage_count += 1; }
}

/// Secure message channel with encryption and authentication
pub struct SecureMessageChannel {
	pub config:SecurityConfig,
	pub current_key:Arc<RwLock<EncryptionKey>>,
	pub previous_keys:Arc<RwLock<HashMap<String, EncryptionKey>>>,
	pub hmac_key:Arc<RwLock<Vec<u8>>>,
	pub rng:SystemRandom,
	pub key_rotation_task:Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl SecureMessageChannel {
	/// Create a new secure message channel
	pub fn new(config:SecurityConfig) -> Result<Self, String> {
		let rng = SystemRandom::new();

		// Generate encryption key
		let mut encryption_key_bytes = vec![0u8; 32];
		rng.fill(&mut encryption_key_bytes)
			.map_err(|e| format!("Failed to generate encryption key: {}", e))?;

		let encryption_key = EncryptionKey::new(&encryption_key_bytes)?;

		// Generate HMAC key
		let mut hmac_key = vec![0u8; 32];
		rng.fill(&mut hmac_key)
			.map_err(|e| format!("Failed to generate HMAC key: {}", e))?;

		let channel = Self {
			config,
			current_key:Arc::new(RwLock::new(encryption_key)),
			previous_keys:Arc::new(RwLock::new(HashMap::new())),
			hmac_key:Arc::new(RwLock::new(hmac_key)),
			rng,
			key_rotation_task:Arc::new(RwLock::new(None)),
		};

		info!(
			"[SecureMessageChannel] Created secure channel with {} encryption",
			channel.config.encryption_algorithm
		);

		Ok(channel)
	}

	/// Start the secure channel with automatic key rotation
	pub async fn start(&self) -> Result<(), String> {
		// Start key rotation task
		self.start_key_rotation().await;

		info!("[SecureMessageChannel] Secure channel started");
		Ok(())
	}

	/// Stop the secure channel
	pub async fn stop(&self) -> Result<(), String> {
		// Stop key rotation task
		{
			let mut rotation_task = self.key_rotation_task.write().await;
			if let Some(task) = rotation_task.take() {
				task.abort();
			}
		}

		// Clear keys
		{
			let mut current_key = self.current_key.write().await;
			*current_key = EncryptionKey::new(&[0u8; 32]).unwrap(); // Zero key
		}

		{
			let mut previous_keys = self.previous_keys.write().await;
			previous_keys.clear();
		}

		{
			let mut hmac_key = self.hmac_key.write().await;
			hmac_key.fill(0); // Zero out HMAC key
		}

		info!("[SecureMessageChannel] Secure channel stopped");
		Ok(())
	}

	/// Encrypt and authenticate a message
	pub async fn encrypt_message<T:Serialize>(&self, message:&T) -> Result<EncryptedMessage, String> {
		// Serialize message
		let serialized_data = encode_to_vec(message, bincode::config::standard())
			.map_err(|e| format!("Failed to serialize message: {}", e))?;

		// Check message size
		if serialized_data.len() > self.config.max_message_size_bytes {
			return Err(format!("Message too large: {} bytes", serialized_data.len()));
		}

		// Get current encryption key
		let mut current_key = self.current_key.write().await;
		current_key.increment_usage();

		// Generate nonce
		let mut nonce = vec![0u8; self.config.nonce_size_bytes];
		self.rng
			.fill(&mut nonce)
			.map_err(|e| format!("Failed to generate nonce: {}", e))?;

		// Encrypt message
		let mut in_out = serialized_data.clone();
		let nonce_slice:&[u8] = &nonce;
		let nonce_array:[u8; NONCE_LEN] = nonce_slice.try_into().map_err(|_| "Invalid nonce length".to_string())?;

		let aead_nonce = aead::Nonce::assume_unique_for_key(nonce_array);

		current_key
			.key
			.seal_in_place_append_tag(aead_nonce, aead::Aad::empty(), &mut in_out)
			.map_err(|e| format!("Encryption failed: {}", e))?;

		// Create HMAC
		let hmac_key = self.hmac_key.read().await;
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &hmac_key);
		let hmac_tag = hmac::sign(&hmac_key, &in_out);

		let encrypted_message = EncryptedMessage {
			key_id:current_key.key_id.clone(),
			nonce:nonce.to_vec(),
			ciphertext:in_out,
			hmac_tag:hmac_tag.as_ref().to_vec(),
			timestamp:SystemTime::now()
				.duration_since(SystemTime::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		};

		trace!(
			"[SecureMessageChannel] Message encrypted (size: {} bytes)",
			encrypted_message.ciphertext.len()
		);

		Ok(encrypted_message)
	}

	/// Decrypt and verify a message
	pub async fn decrypt_message<T:for<'de> Deserialize<'de>>(&self, encrypted:&EncryptedMessage) -> Result<T, String> {
		// Verify HMAC
		let hmac_key = self.hmac_key.read().await;
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &hmac_key);

		hmac::verify(&hmac_key, &encrypted.ciphertext, &encrypted.hmac_tag)
			.map_err(|_| "HMAC verification failed".to_string())?;

		// Get encryption key
		let encryption_key = self.get_encryption_key(&encrypted.key_id).await?;

		// Decrypt message
		let mut in_out = encrypted.ciphertext.clone();
		let nonce_slice:&[u8] = &encrypted.nonce;
		let nonce_array:[u8; NONCE_LEN] = nonce_slice.try_into().map_err(|_| "Invalid nonce length".to_string())?;

		let nonce = aead::Nonce::assume_unique_for_key(nonce_array);

		encryption_key
			.key
			.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
			.map_err(|e| format!("Decryption failed: {}", e))?;

		// Remove authentication tag
		let plaintext_len = in_out.len() - AES_256_GCM.tag_len();
		in_out.truncate(plaintext_len);

		// Deserialize message
		let (message, _) = decode_from_slice(&in_out, bincode::config::standard())
			.map_err(|e| format!("Failed to deserialize message: {}", e))?;

		trace!("[SecureMessageChannel] Message decrypted successfully");

		Ok(message)
	}

	/// Rotate encryption keys
	pub async fn rotate_keys(&self) -> Result<(), String> {
		info!("[SecureMessageChannel] Rotating encryption keys");

		// Generate new encryption key
		let mut new_key_bytes = vec![0u8; 32];
		self.rng
			.fill(&mut new_key_bytes)
			.map_err(|e| format!("Failed to generate new encryption key: {}", e))?;

		let new_key = EncryptionKey::new(&new_key_bytes)?;

		// Move current key to previous keys
		{
			let mut current_key = self.current_key.write().await;
			let mut previous_keys = self.previous_keys.write().await;

			previous_keys.insert(current_key.key_id.clone(), current_key.clone());
			*current_key = new_key;
		}

		// Clean up old keys
		self.cleanup_old_keys().await;

		debug!("[SecureMessageChannel] Key rotation completed");
		Ok(())
	}

	/// Get encryption key by ID
	async fn get_encryption_key(&self, key_id:&str) -> Result<EncryptionKey, String> {
		// Check current key first
		let current_key = self.current_key.read().await;
		if current_key.key_id == key_id {
			return Ok(current_key.clone());
		}

		// Check previous keys
		let previous_keys = self.previous_keys.read().await;
		if let Some(key) = previous_keys.get(key_id) {
			return Ok(key.clone());
		}

		Err(format!("Encryption key not found: {}", key_id))
	}

	/// Start automatic key rotation
	async fn start_key_rotation(&self) {
		let channel = Arc::new(self.clone());

		let rotation_interval = Duration::from_secs(self.config.key_rotation_interval_hours * 3600);

		let task = tokio::spawn(async move {
			let mut interval = tokio::time::interval(rotation_interval);

			loop {
				interval.tick().await;

				if let Err(e) = channel.rotate_keys().await {
					error!("[SecureMessageChannel] Automatic key rotation failed: {}", e);
				}
			}
		});

		{
			let mut rotation_task = self.key_rotation_task.write().await;
			*rotation_task = Some(task);
		}
	}

	/// Cleanup old keys
	async fn cleanup_old_keys(&self) {
		let rotation_interval = Duration::from_secs(self.config.key_rotation_interval_hours * 3600);
		let max_age = rotation_interval * 2; // Keep keys for 2 rotation cycles

		let mut previous_keys = self.previous_keys.write().await;

		previous_keys.retain(|_, key| !key.is_expired(max_age));

		debug!("[SecureMessageChannel] Cleaned up {} old keys", previous_keys.len());
	}

	/// Get security statistics
	pub async fn get_stats(&self) -> SecurityStats {
		let current_key = self.current_key.read().await;
		let previous_keys = self.previous_keys.read().await;

		SecurityStats {
			current_key_id:current_key.key_id.clone(),
			current_key_age_seconds:current_key.created_at.elapsed().unwrap_or_default().as_secs(),
			current_key_usage_count:current_key.usage_count,
			previous_keys_count:previous_keys.len(),
			config:self.config.clone(),
		}
	}

	/// Validate message integrity
	pub async fn validate_message_integrity(&self, encrypted:&EncryptedMessage) -> Result<bool, String> {
		// Check timestamp (prevent replay attacks)
		let message_time = SystemTime::UNIX_EPOCH + Duration::from_millis(encrypted.timestamp);
		let current_time = SystemTime::now();

		if current_time.duration_since(message_time).unwrap_or_default() > Duration::from_secs(300) {
			// Message is older than 5 minutes
			return Ok(false);
		}

		// Verify HMAC
		let hmac_key = self.hmac_key.read().await;
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &hmac_key);

		match hmac::verify(&hmac_key, &encrypted.ciphertext, &encrypted.hmac_tag) {
			Ok(_) => Ok(true),
			Err(_) => Ok(false),
		}
	}

	/// Create a secure channel with default configuration
	pub fn default_channel() -> Result<Self, String> { Self::new(SecurityConfig::default()) }

	/// Create a high-security channel
	pub fn high_security_channel() -> Result<Self, String> {
		Self::new(SecurityConfig {
			key_rotation_interval_hours:1,          // Rotate keys hourly
			max_message_size_bytes:1 * 1024 * 1024, // 1MB
			..Default::default()
		})
	}
}

impl Clone for SecureMessageChannel {
	fn clone(&self) -> Self {
		Self {
			config:self.config.clone(),
			current_key:self.current_key.clone(),
			previous_keys:self.previous_keys.clone(),
			hmac_key:self.hmac_key.clone(),
			rng:SystemRandom::new(),
			key_rotation_task:Arc::new(RwLock::new(None)),
		}
	}
}

/// Encrypted message structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
	pub key_id:String,
	pub nonce:Vec<u8>,
	pub ciphertext:Vec<u8>,
	pub hmac_tag:Vec<u8>,
	pub timestamp:u64,
}

/// Security statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityStats {
	pub current_key_id:String,
	pub current_key_age_seconds:u64,
	pub current_key_usage_count:usize,
	pub previous_keys_count:usize,
	pub config:SecurityConfig,
}

/// Utility functions for secure messaging
impl SecureMessageChannel {
	/// Generate a secure random key
	pub fn generate_secure_key(key_size_bytes:usize) -> Result<Vec<u8>, String> {
		let rng = SystemRandom::new();
		let mut key = vec![0u8; key_size_bytes];

		rng.fill(&mut key)
			.map_err(|e| format!("Failed to generate secure key: {}", e))?;

		Ok(key)
	}

	/// Calculate message overhead for encryption
	pub fn calculate_encryption_overhead(message_size:usize) -> usize {
		// Nonce + HMAC tag + encryption overhead
		NONCE_LEN + AES_256_GCM.tag_len() + 16 // Additional padding
	}

	/// Estimate encrypted message size
	pub fn estimate_encrypted_size(original_size:usize) -> usize {
		original_size + Self::calculate_encryption_overhead(original_size)
	}

	/// Create message with secure headers
	pub async fn create_secure_message<T:Serialize>(
		&self,
		message:&T,
		additional_headers:HashMap<String, String>,
	) -> Result<SecureMessage<T>, String> {
		let encrypted = self.encrypt_message(message).await?;

		Ok(SecureMessage::<T> {
			encrypted,
			headers:additional_headers,
			version:"1.0".to_string(),
			_marker:PhantomData,
		})
	}
}

/// Secure message with headers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureMessage<T> {
	pub encrypted:EncryptedMessage,
	pub headers:HashMap<String, String>,
	pub version:String,
	#[serde(skip)]
	_marker:PhantomData<T>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[tokio::test]
	async fn test_secure_channel_creation() {
		let channel = SecureMessageChannel::default_channel().unwrap();
		assert_eq!(channel.config.encryption_algorithm, "AES-256-GCM");
	}

	#[tokio::test]
	async fn test_message_encryption_decryption() {
		let channel = SecureMessageChannel::default_channel().unwrap();
		channel.start().await.unwrap();

		let test_message = "Hello, secure world!";
		let encrypted = channel.encrypt_message(&test_message).await.unwrap();

		assert!(!encrypted.ciphertext.is_empty());
		assert!(!encrypted.hmac_tag.is_empty());
		assert!(!encrypted.nonce.is_empty());

		let decrypted:String = channel.decrypt_message(&encrypted).await.unwrap();
		assert_eq!(decrypted, test_message);

		channel.stop().await.unwrap();
	}

	#[tokio::test]
	async fn test_message_validation() {
		let channel = SecureMessageChannel::default_channel().unwrap();
		channel.start().await.unwrap();

		let test_message = "Test validation";
		let encrypted = channel.encrypt_message(&test_message).await.unwrap();

		let is_valid = channel.validate_message_integrity(&encrypted).await.unwrap();
		assert!(is_valid);

		channel.stop().await.unwrap();
	}

	#[tokio::test]
	async fn test_key_rotation() {
		let channel = SecureMessageChannel::default_channel().unwrap();
		channel.start().await.unwrap();

		let stats_before = channel.get_stats().await;

		// Rotate keys
		channel.rotate_keys().await.unwrap();

		let stats_after = channel.get_stats().await;
		assert_ne!(stats_before.current_key_id, stats_after.current_key_id);
		assert_eq!(stats_after.previous_keys_count, 1);

		channel.stop().await.unwrap();
	}

	#[test]
	fn test_secure_key_generation() {
		let key = SecureMessageChannel::generate_secure_key(32).unwrap();
		assert_eq!(key.len(), 32);
	}

	#[test]
	fn test_encryption_overhead_calculation() {
		let overhead = SecureMessageChannel::calculate_encryption_overhead(100);
		assert!(overhead > 0);

		let estimated_size = SecureMessageChannel::estimate_encrypted_size(100);
		assert!(estimated_size > 100);
	}
}
