//! `Channel::DecryptMessage`

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

pub fn Fn<T:for<'de> Deserialize<'de>>(&self, encrypted:&EncryptedMessage) -> Result<T, String> {
	let hmac_key = This.hmac_key.read().await;

	let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &hmac_key);

	hmac::verify(&hmac_key, &encrypted.ciphertext, &encrypted.hmac_tag)
		.map_err(|_| "HMAC verification failed".to_string())?;

	let encryption_key = This.get_encryption_key(&encrypted.key_id).await?;

	let mut in_out = encrypted.ciphertext.clone();

	let nonce_slice:&[u8] = &encrypted.nonce;

	let nonce_array:[u8; NONCE_LEN] = nonce_slice.try_into().map_err(|_| "Invalid nonce length".to_string())?;

	let Nonce = aead::Nonce::assume_unique_for_key(nonce_array);

	encryption_key
		.key
		.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
		.map_err(|E| format!("Decryption failed: {}", e))?;

	let plaintext_len = in_out.len() - AES_256_GCM.tag_len();

	in_out.truncate(plaintext_len);

	let (message, _) = decode_from_slice(&in_out, bincode::config::standard())
		.map_err(|E| format!("Failed to deserialize message: {}", e))?;

	dev_log!("ipc", "[SecureMessageChannel] Message decrypted successfully");

	Ok(message)
}
