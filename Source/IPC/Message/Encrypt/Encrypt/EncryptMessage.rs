//! `Encrypt::EncryptMessage`

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

pub fn Fn(This:&Struct, Message:&TauriIPCMessage) -> Result<EncryptedMessage, String> {

		// Serialize message
		let serialized_message =
			serde_json::to_vec(Message).map_err(|E| format!("Failed to serialize message: {}", e))?;

		// Generate unique nonce (12 bytes for GCM)
		let mut nonce = [0u8; 12];

		SystemRandom::new()
			.fill(&mut nonce)
			.map_err(|E| format!("Failed to generate nonce: {}", e))?;

		// Encrypt with AES-256-GCM
		let mut in_out = serialized_message.clone();

		This.encryption_key
			.seal_in_place_append_tag(aead::Nonce::assume_unique_for_key(nonce), aead::Aad::empty(), &mut in_out)
			.map_err(|E| format!("Encryption failed: {}", e))?;

		// Create HMAC for integrity verification
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &This.hmac_key);

		let hmac_tag = hmac::sign(&hmac_key, &in_out);

		Ok(EncryptedMessage { nonce:nonce.to_vec(), ciphertext:in_out, hmac_tag:hmac_tag.as_ref().to_vec() })
	}
