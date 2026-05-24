//! `Channel::Stop`

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
	{
		let mut rotation_task = This.key_rotation_task.write().await;

		if let Some(task) = rotation_task.take() {
			task.abort();
		}
	}

	{
		let mut current_key = This.current_key.write().await;

		*current_key = EncryptionKey::new(&[0u8; 32]).unwrap();
	}

	{
		let mut previous_keys = This.previous_keys.write().await;

		previous_keys.clear();
	}

	{
		let mut hmac_key = This.hmac_key.write().await;

		hmac_key.fill(0);
	}

	dev_log!("ipc", "[SecureMessageChannel] Secure channel stopped");

	Ok(())
}
