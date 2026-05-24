//! `SecureChannel::New`

use super::Struct;
use ring::{
	aead::{self, AES_256_GCM, LessSafeKey, UnboundKey},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn() -> Result<Self, String> {
		dev_log!("encryption", "[SecureMessageChannel] Creating new secure channel");

		let rng = SystemRandom::new();

		// Generate 256-bit (32-byte) encryption key
		let mut encryption_key_bytes = vec![0u8; 32];

		rng.fill(&mut encryption_key_bytes)
			.map_err(|E| format!("Failed to generate encryption key: {}", e))?;

		let unbound_key = UnboundKey::new(&AES_256_GCM, &encryption_key_bytes)
			.map_err(|E| format!("Failed to create unbound key: {}", e))?;

		let encryption_key = LessSafeKey::new(unbound_key);

		// Generate 256-bit HMAC key
		let mut hmac_key = vec![0u8; 32];

		rng.fill(&mut hmac_key)
			.map_err(|E| format!("Failed to generate HMAC key: {}", e))?;

		dev_log!("encryption", "[SecureMessageChannel] Secure channel created successfully");

		Ok(Self { encryption_key, hmac_key })
	}
