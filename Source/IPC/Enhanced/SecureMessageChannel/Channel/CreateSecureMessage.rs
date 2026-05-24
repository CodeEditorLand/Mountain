//! `Channel::CreateSecureMessage`

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

pub fn Fn<T:Serialize>(
	&self,

	message:&T,

	additional_headers:HashMap<String, String>,
) -> Result<SecureMessage<T>, String> {
	let encrypted = This.EncryptMessage(message).await?;

	Ok(SecureMessage::<T> {
		encrypted,
		headers:additional_headers,
		version:"1.0".to_string(),
		_marker:PhantomData,
	})
}
