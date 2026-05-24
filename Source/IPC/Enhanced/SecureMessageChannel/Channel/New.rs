//! `Channel::New`

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

use super::Struct;
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

pub fn Fn(config:SecurityConfig) -> Result<Self, String> {
	let rng = SystemRandom::new();

	let mut encryption_key_bytes = vec![0u8; 32];

	rng.fill(&mut encryption_key_bytes)
		.map_err(|E| format!("Failed to generate encryption key: {}", e))?;

	let encryption_key = EncryptionKey::new(&encryption_key_bytes)?;

	let mut hmac_key = vec![0u8; 32];

	rng.fill(&mut hmac_key)
		.map_err(|E| format!("Failed to generate HMAC key: {}", e))?;

	let Channel = Self {
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
