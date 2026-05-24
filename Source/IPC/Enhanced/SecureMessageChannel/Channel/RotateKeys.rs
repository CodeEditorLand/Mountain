//! `Channel::RotateKeys`

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

pub fn Fn(This:&Struct) -> Result<(), String> {
	dev_log!("ipc", "[SecureMessageChannel] Rotating encryption keys");

	let mut new_key_bytes = vec![0u8; 32];

	This.rng
		.fill(&mut new_key_bytes)
		.map_err(|E| format!("Failed to generate new encryption key: {}", e))?;

	let new_key = EncryptionKey::new(&new_key_bytes)?;

	{
		let mut current_key = This.current_key.write().await;

		let mut previous_keys = This.previous_keys.write().await;

		previous_keys.insert(current_key.key_id.clone(), current_key.clone());

		*current_key = new_key;
	}

	This.cleanup_old_keys().await;

	dev_log!("ipc", "[SecureMessageChannel] Key rotation completed");

	Ok(())
}
