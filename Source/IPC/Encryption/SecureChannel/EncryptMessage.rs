//! `SecureChannel::EncryptMessage`

use super::Struct;
use ring::{
	aead::{self, AES_256_GCM, LessSafeKey, UnboundKey},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct, message:&TauriIPCMessage) -> Result<EncryptedMessage, String> {
		dev_log!(
			"encryption",
			"[SecureMessageChannel] Encrypting message on channel: {}",
			message.channel
		);

		// Serialize message to bytes
		let serialized_message =
			serde_json::to_vec(message).map_err(|E| format!("Failed to serialize message: {}", e))?;

		// Generate unique 12-byte nonce (required for AES-256-GCM)
		let mut nonce = [0u8; 12];

		SystemRandom::new()
			.fill(&mut nonce)
			.map_err(|E| format!("Failed to generate nonce: {}", e))?;

		// Encrypt with AES-256-GCM (authenticated encryption)
		let mut in_out = serialized_message.clone();

		This.encryption_key
			.seal_in_place_append_tag(aead::Nonce::assume_unique_for_key(nonce), aead::Aad::empty(), &mut in_out)
			.map_err(|E| format!("Encryption failed: {}", e))?;

		// Generate HMAC for additional authentication
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &This.hmac_key);

		let hmac_tag = hmac::sign(&hmac_key, &in_out);

		let encrypted_message =
			EncryptedMessage { nonce:nonce.to_vec(), ciphertext:in_out, hmac_tag:hmac_tag.as_ref().to_vec() };

		dev_log!(
			"encryption",
			"[SecureMessageChannel] Message encrypted: {} bytes -> {} bytes",
			serialized_message.len(),
			encrypted_message.ciphertext.len()
		);

		Ok(encrypted_message)
	}
