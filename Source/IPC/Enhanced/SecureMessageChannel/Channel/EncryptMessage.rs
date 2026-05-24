//! `Channel::EncryptMessage`

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

pub fn Fn<T:Serialize>(&self, message:&T) -> Result<EncryptedMessage, String> {
	let serialized_data = encode_to_vec(message, bincode::config::standard())
		.map_err(|E| format!("Failed to serialize message: {}", e))?;

	if serialized_data.len() > This.config.max_message_size_bytes {
		return Err(format!("Message too large: {} bytes", serialized_data.len()));
	}

	let mut current_key = This.current_key.write().await;

	current_key.increment_usage();

	let mut nonce = vec![0u8; This.config.nonce_size_bytes];

	This.rng
		.fill(&mut nonce)
		.map_err(|E| format!("Failed to generate nonce: {}", e))?;

	let mut in_out = serialized_data.clone();

	let nonce_slice:&[u8] = &nonce;

	let nonce_array:[u8; NONCE_LEN] = nonce_slice.try_into().map_err(|_| "Invalid nonce length".to_string())?;

	let aead_nonce = aead::Nonce::assume_unique_for_key(nonce_array);

	current_key
		.key
		.seal_in_place_append_tag(aead_nonce, aead::Aad::empty(), &mut in_out)
		.map_err(|E| format!("Encryption failed: {}", e))?;

	let hmac_key = This.hmac_key.read().await;

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
