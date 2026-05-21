#![allow(non_snake_case)]

//! `encryption:decrypt(value: string) -> string`
//!
//! Reverses `encryption:encrypt`: base64-decodes, splits the 12-byte nonce
//! from the ciphertext+tag, decrypts with AES-256-GCM, and returns the
//! original plaintext string. Returns an empty string on any failure so the
//! workbench treats a corrupt blob as "no stored secret" rather than crashing.

use base64::{Engine, engine::general_purpose::STANDARD as B64};
use ring::aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey};
use serde_json::{Value, json};

use crate::dev_log;
use super::super::Encryption::Key::DeriveKey;

pub async fn Decrypt(Arguments:Vec<Value>) -> Result<Value, String> {
	let Ciphertext = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	if Ciphertext.is_empty() {
		return Ok(json!(""));
	}

	let Blob = match B64.decode(&Ciphertext) {
		Ok(B) => B,

		Err(_) => {
			dev_log!("encryption", "warn: encryption:decrypt invalid base64 - returning empty");

			return Ok(json!(""));
		},
	};

	// Minimum: 12 (nonce) + 16 (GCM tag) = 28 bytes
	if Blob.len() < 28 {
		dev_log!("encryption", "warn: encryption:decrypt blob too short ({} bytes)", Blob.len());

		return Ok(json!(""));
	}

	let KeyBytes = DeriveKey().map_err(|E| format!("encryption:decrypt unavailable - {E}"))?;

	let UnboundK = match UnboundKey::new(&AES_256_GCM, &KeyBytes) {
		Ok(K) => K,

		Err(_) => return Ok(json!("")),
	};

	let Key = LessSafeKey::new(UnboundK);

	let NonceBytes:[u8; 12] = Blob[..12].try_into().unwrap();

	let NonceVal = Nonce::assume_unique_for_key(NonceBytes);

	let mut Data = Blob[12..].to_vec();

	match Key.open_in_place(NonceVal, Aad::empty(), &mut Data) {
		Ok(Plaintext) => {
			let S = String::from_utf8_lossy(Plaintext).into_owned();

			dev_log!("encryption", "encryption:decrypt ok ({} bytes)", S.len());

			Ok(json!(S))
		},

		Err(_) => {
			dev_log!(
				"encryption",
				"warn: encryption:decrypt open_in_place failed (wrong key or corrupt)"
			);

			Ok(json!(""))
		},
	}
}
