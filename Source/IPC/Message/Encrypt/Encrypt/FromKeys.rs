//! `Encrypt::FromKeys`

use super::Struct;
use std::array::TryFromSliceError;
use base64::{Engine, engine::general_purpose};
use ring::{
	aead,
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use super::super::Define::DefineMessage::TauriIPCMessage;

pub fn Fn(encryption_key_bytes:&[u8], hmac_key:&[u8]) -> Result<Self, String> {

		if encryption_key_bytes.len() != 32 {

			return Err(format!(
				"Invalid encryption key length: {} (expected 32)",

				encryption_key_bytes.len()
			));
		}

		if hmac_key.len() != 32 {

			return Err(format!("Invalid HMAC key length: {} (expected 32)", hmac_key.len()));
		}

		let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, encryption_key_bytes)
			.map_err(|E| format!("Failed to create unbound key: {}", e))?;

		Ok(Self { encryption_key:aead::LessSafeKey::new(unbound_key), hmac_key:hmac_key.to_vec() })
	}
