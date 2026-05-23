
//! `Channel::Struct` - AES-256-GCM + HMAC-SHA256 secure
//! message channel with automatic key rotation and replay
//! protection. The struct + 18-method impl + Clone + utility
//! impl stay in one file - tightly coupled cluster.

use std::{
	collections::HashMap,
	marker::PhantomData,
	sync::Arc,
	time::{Duration, SystemTime},
};

use bincode::serde::{decode_from_slice, encode_to_vec};
use ring::{
	aead::{self, AES_256_GCM, NONCE_LEN},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{
	IPC::Enhanced::SecureMessageChannel::{
		EncryptedMessage::Struct as EncryptedMessage,
		EncryptionKey::Struct as EncryptionKey,
		SecureMessage::Struct as SecureMessage,
		SecurityConfig::Struct as SecurityConfig,
		SecurityStats::Struct as SecurityStats,
	},
	dev_log,
};

pub struct Struct {
	pub config:SecurityConfig,

	pub current_key:Arc<RwLock<EncryptionKey>>,

	pub previous_keys:Arc<RwLock<HashMap<String, EncryptionKey>>>,

	pub hmac_key:Arc<RwLock<Vec<u8>>>,

	pub rng:SystemRandom,

	pub key_rotation_task:Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl Struct {
	pub fn new(config:SecurityConfig) -> Result<Self, String> {
		let rng = SystemRandom::new();

		let mut encryption_key_bytes = vec![0u8; 32];

		rng.fill(&mut encryption_key_bytes)
			.map_err(|e| format!("Failed to generate encryption key: {}", e))?;

		let encryption_key = EncryptionKey::new(&encryption_key_bytes)?;

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

		dev_log!(
			"ipc",
			"[SecureMessageChannel] Created secure channel with {} encryption",
			channel.config.encryption_algorithm
		);

		Ok(channel)
	}

	pub async fn start(&self) -> Result<(), String> {
		self.start_key_rotation().await;

		dev_log!("ipc", "[SecureMessageChannel] Secure channel started");

		Ok(())
	}

	pub async fn stop(&self) -> Result<(), String> {
		{
			let mut rotation_task = self.key_rotation_task.write().await;

			if let Some(task) = rotation_task.take() {
				task.abort();
			}
		}

		{
			let mut current_key = self.current_key.write().await;

			*current_key = EncryptionKey::new(&[0u8; 32]).unwrap();
		}

		{
			let mut previous_keys = self.previous_keys.write().await;

			previous_keys.clear();
		}

		{
			let mut hmac_key = self.hmac_key.write().await;

			hmac_key.fill(0);
		}

		dev_log!("ipc", "[SecureMessageChannel] Secure channel stopped");

		Ok(())
	}

	pub async fn encrypt_message<T:Serialize>(&self, message:&T) -> Result<EncryptedMessage, String> {
		let serialized_data = encode_to_vec(message, bincode::config::standard())
			.map_err(|e| format!("Failed to serialize message: {}", e))?;

		if serialized_data.len() > self.config.max_message_size_bytes {
			return Err(format!("Message too large: {} bytes", serialized_data.len()));
		}

		let mut current_key = self.current_key.write().await;

		current_key.increment_usage();

		let mut nonce = vec![0u8; self.config.nonce_size_bytes];

		self.rng
			.fill(&mut nonce)
			.map_err(|e| format!("Failed to generate nonce: {}", e))?;

		let mut in_out = serialized_data.clone();

		let nonce_slice:&[u8] = &nonce;

		let nonce_array:[u8; NONCE_LEN] = nonce_slice.try_into().map_err(|_| "Invalid nonce length".to_string())?;

		let aead_nonce = aead::Nonce::assume_unique_for_key(nonce_array);

		current_key
			.key
			.seal_in_place_append_tag(aead_nonce, aead::Aad::empty(), &mut in_out)
			.map_err(|e| format!("Encryption failed: {}", e))?;

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

		dev_log!(
			"ipc",
			"[SecureMessageChannel] Message encrypted (size: {} bytes)",
			encrypted_message.ciphertext.len()
		);

		Ok(encrypted_message)
	}

	pub async fn decrypt_message<T:for<'de> Deserialize<'de>>(&self, encrypted:&EncryptedMessage) -> Result<T, String> {
		let hmac_key = self.hmac_key.read().await;

		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &hmac_key);

		hmac::verify(&hmac_key, &encrypted.ciphertext, &encrypted.hmac_tag)
			.map_err(|_| "HMAC verification failed".to_string())?;

		let encryption_key = self.get_encryption_key(&encrypted.key_id).await?;

		let mut in_out = encrypted.ciphertext.clone();

		let nonce_slice:&[u8] = &encrypted.nonce;

		let nonce_array:[u8; NONCE_LEN] = nonce_slice.try_into().map_err(|_| "Invalid nonce length".to_string())?;

		let nonce = aead::Nonce::assume_unique_for_key(nonce_array);

		encryption_key
			.key
			.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
			.map_err(|e| format!("Decryption failed: {}", e))?;

		let plaintext_len = in_out.len() - AES_256_GCM.tag_len();

		in_out.truncate(plaintext_len);

		let (message, _) = decode_from_slice(&in_out, bincode::config::standard())
			.map_err(|e| format!("Failed to deserialize message: {}", e))?;

		dev_log!("ipc", "[SecureMessageChannel] Message decrypted successfully");

		Ok(message)
	}

	pub async fn rotate_keys(&self) -> Result<(), String> {
		dev_log!("ipc", "[SecureMessageChannel] Rotating encryption keys");

		let mut new_key_bytes = vec![0u8; 32];

		self.rng
			.fill(&mut new_key_bytes)
			.map_err(|e| format!("Failed to generate new encryption key: {}", e))?;

		let new_key = EncryptionKey::new(&new_key_bytes)?;

		{
			let mut current_key = self.current_key.write().await;

			let mut previous_keys = self.previous_keys.write().await;

			previous_keys.insert(current_key.key_id.clone(), current_key.clone());

			*current_key = new_key;
		}

		self.cleanup_old_keys().await;

		dev_log!("ipc", "[SecureMessageChannel] Key rotation completed");

		Ok(())
	}

	async fn get_encryption_key(&self, key_id:&str) -> Result<EncryptionKey, String> {
		let current_key = self.current_key.read().await;

		if current_key.key_id == key_id {
			return Ok(current_key.clone());
		}

		let previous_keys = self.previous_keys.read().await;

		if let Some(key) = previous_keys.get(key_id) {
			return Ok(key.clone());
		}

		Err(format!("Encryption key not found: {}", key_id))
	}

	async fn start_key_rotation(&self) {
		let channel = Arc::new(self.clone());

		let rotation_interval = Duration::from_secs(self.config.key_rotation_interval_hours * 3600);

		let task = tokio::spawn(async move {
			let mut interval = tokio::time::interval(rotation_interval);

			loop {
				interval.tick().await;

				if let Err(e) = channel.rotate_keys().await {
					dev_log!("ipc", "error: [SecureMessageChannel] Automatic key rotation failed: {}", e);
				}
			}
		});

		{
			let mut rotation_task = self.key_rotation_task.write().await;

			*rotation_task = Some(task);
		}
	}

	async fn cleanup_old_keys(&self) {
		let rotation_interval = Duration::from_secs(self.config.key_rotation_interval_hours * 3600);

		let max_age = rotation_interval * 2;

		let mut previous_keys = self.previous_keys.write().await;

		previous_keys.retain(|_, key| !key.is_expired(max_age));

		dev_log!("ipc", "[SecureMessageChannel] Cleaned up {} old keys", previous_keys.len());
	}

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

	pub async fn validate_message_integrity(&self, encrypted:&EncryptedMessage) -> Result<bool, String> {
		let message_time = SystemTime::UNIX_EPOCH + Duration::from_millis(encrypted.timestamp);

		let current_time = SystemTime::now();

		if current_time.duration_since(message_time).unwrap_or_default() > Duration::from_secs(300) {
			return Ok(false);
		}

		let hmac_key = self.hmac_key.read().await;

		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &hmac_key);

		match hmac::verify(&hmac_key, &encrypted.ciphertext, &encrypted.hmac_tag) {
			Ok(_) => Ok(true),

			Err(_) => Ok(false),
		}
	}

	pub fn default_channel() -> Result<Self, String> { Self::new(SecurityConfig::default()) }

	pub fn high_security_channel() -> Result<Self, String> {
		Self::new(SecurityConfig {
			key_rotation_interval_hours:1,
			max_message_size_bytes:1024 * 1024,
			..Default::default()
		})
	}

	pub fn generate_secure_key(key_size_bytes:usize) -> Result<Vec<u8>, String> {
		let rng = SystemRandom::new();

		let mut key = vec![0u8; key_size_bytes];

		rng.fill(&mut key)
			.map_err(|e| format!("Failed to generate secure key: {}", e))?;

		Ok(key)
	}

	pub fn calculate_encryption_overhead(_message_size:usize) -> usize { NONCE_LEN + AES_256_GCM.tag_len() + 16 }

	pub fn estimate_encrypted_size(original_size:usize) -> usize {
		original_size + Self::calculate_encryption_overhead(original_size)
	}

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

impl Clone for Struct {
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
