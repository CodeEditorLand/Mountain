//! `Encrypt::DecryptMessage`

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

pub fn Fn(This:&Struct, Encrypted:&EncryptedMessage) -> Result<TauriIPCMessage, String> {

		// Validate structure
		Encrypted.Validate()?;

		// Verify HMAC first (fast-fail)
		let hmac_key = hmac::Key::new(hmac::HMAC_SHA256, &This.hmac_key);

		hmac::verify(&hmac_key, &Encrypted.ciphertext, &Encrypted.hmac_tag)
			.map_err(|_| "HMAC verification failed - message may be tampered".to_string())?;

		// Decrypt with AES-256-GCM
		let mut in_out = Encrypted.ciphertext.clone();

		let nonce_slice:&[u8] = &Encrypted.nonce;

		let nonce_array:[u8; 12] = nonce_slice
			.try_into()
			.map_err(|_:TryFromSliceError| "Invalid nonce length".to_string())?;

		let Nonce = aead::Nonce::assume_unique_for_key(nonce_array);

		This.encryption_key
			.open_in_place(nonce, aead::Aad::empty(), &mut in_out)
			.map_err(|E| format!("Decryption failed: {}", e))?;

		// Remove authentication tag
		const TAG_LEN:usize = 16;

		let plaintext_len = in_out.len() - TAG_LEN;

		in_out.truncate(plaintext_len);

		// Deserialize message
		serde_json::from_slice(&in_out).map_err(|E| format!("Failed to deserialize message: {}", e))
	}
