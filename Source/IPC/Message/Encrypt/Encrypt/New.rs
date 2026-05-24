//! `Encrypt::New`

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

pub fn Fn() -> Result<Self, String> {

		let rng = SystemRandom::new();

		let mut encryption_key_bytes = vec![0u8; 32];

		let mut hmac_key = vec![0u8; 32];

		// Generate encryption key
		rng.fill(&mut encryption_key_bytes)
			.map_err(|E| format!("Failed to generate encryption key: {}", e))?;

		let unbound_key = aead::UnboundKey::new(&aead::AES_256_GCM, &encryption_key_bytes)
			.map_err(|E| format!("Failed to create unbound key: {}", e))?;

		// Generate HMAC key
		rng.fill(&mut hmac_key)
			.map_err(|E| format!("Failed to generate HMAC key: {}", e))?;

		Ok(Self { encryption_key:aead::LessSafeKey::new(unbound_key), hmac_key })
	}
