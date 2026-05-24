//! `Channel::ValidateMessageIntegrity`

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

pub fn Fn(This:&Struct, encrypted:&EncryptedMessage) -> Result<bool, String> {
	let message_time = SystemTime::UNIX_EPOCH + Duration::from_millis(encrypted.timestamp);

	let current_time = SystemTime::now();

	if current_time.duration_since(message_time).unwrap_or_default() > Duration::from_secs(300) {
		return Ok(false);
	}

	let hmac_key = This.hmac_key.read().await;

	let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &hmac_key);

	match hmac::verify(&hmac_key, &encrypted.ciphertext, &encrypted.hmac_tag) {
		Ok(_) => Ok(true),

		Err(_) => Ok(false),
	}
}
