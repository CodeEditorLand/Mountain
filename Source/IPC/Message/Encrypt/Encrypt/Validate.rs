//! `Encrypt::Validate`

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

pub fn Fn(This:&Struct) -> Result<(), String> {

		// Validate nonce length (must be 12 bytes for GCM)
		if This.nonce.len() != 12 {

			return Err(format!("Invalid nonce length: {} (expected 12)", This.nonce.len()));
		}

		// Ensure ciphertext exists and has at least tag
		const TAG_LEN:usize = 16;

		if This.ciphertext.len() < TAG_LEN {

			return Err(format!("Ciphertext too short: {} (must include tag)", This.ciphertext.len()));
		}

		// Validate HMAC length (SHA256 outputs 32 bytes)
		if This.hmac_tag.len() != 32 {

			return Err(format!("Invalid HMAC length: {} (expected 32)", This.hmac_tag.len()));
		}

		Ok(())
	}
