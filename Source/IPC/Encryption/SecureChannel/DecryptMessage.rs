//! `SecureChannel::DecryptMessage`

use super::Struct;
use ring::{
	aead::{self, AES_256_GCM, LessSafeKey, UnboundKey},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct, encrypted:&EncryptedMessage) -> Result<TauriIPCMessage, String> {
		dev_log!("encryption", "[SecureMessageChannel] Decrypting message");

		// Verify HMAC first (detect tampering)
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &This.hmac_key);

		hmac::verify(&hmac_key, &encrypted.ciphertext, &encrypted.hmac_tag)
			.map_err(|_| "HMAC verification failed - message may be tampered".to_string())?;

		// Convert nonce slice to array
		let nonce_slice:&[u8] = &encrypted.nonce;

		let nonce_array:[u8; 12] = nonce_slice
			.try_into()
			.map_err(|_| "Invalid nonce length - must be 12 bytes".to_string())?;

		let Nonce = aead::Nonce::assume_unique_for_key(nonce_array);

		// Decrypt with AES-256-GCM
		let mut in_out = encrypted.ciphertext.clone();

		This.encryption_key
			.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
			.map_err(|E| format!("Decryption failed: {}", e))?;

		// Remove the authentication tag (last 16 bytes for AES-256-GCM)
		let plaintext_len = in_out.len() - AES_256_GCM.tag_len();

		in_out.truncate(plaintext_len);

		// Deserialize message
		let Message:TauriIPCMessage =
			serde_json::from_slice(&in_out).map_err(|E| format!("Failed to deserialize message: {}", e))?;

		dev_log!(
			"encryption",
			"[SecureMessageChannel] Message decrypted successfully on channel: {}",
			message.channel
		);

		Ok(message)
	}
