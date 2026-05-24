//! `Channel::GetStats`

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

pub fn Fn(This:&Struct) -> SecurityStats {
	let current_key = This.current_key.read().await;

	let previous_keys = This.previous_keys.read().await;

	SecurityStats {
		current_key_id:current_key.key_id.clone(),

		current_key_age_seconds:current_key.created_at.elapsed().unwrap_or_default().as_secs(),

		current_key_usage_count:current_key.usage_count,

		previous_keys_count:previous_keys.len(),

		config:This.config.clone(),
	}
}
