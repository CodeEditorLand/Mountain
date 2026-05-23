#![allow(non_snake_case)]

//! `encryption:encrypt(value: string) -> string`
//!
//! Encrypts a plaintext string with AES-256-GCM and returns a base64-encoded
//! `<12-byte nonce><ciphertext+tag>` blob that `encryption:decrypt` can
//! reverse. Called by VS Code's `EncryptionMainService` to store extension
//! secrets and auth tokens safely at rest.

use base64::{Engine, engine::general_purpose::STANDARD as B64};
use ring::{
	aead::{AES_256_GCM, Aad, LessSafeKey, Nonce, UnboundKey},
	rand::{SecureRandom, SystemRandom},
};
use serde_json::{Value, json};

use crate::dev_log;
use super::Key::Fn as DeriveKey;

pub async fn Fn(Arguments:Vec<Value>) -> Result<Value, String> {
	let Plaintext = Arguments.first().and_then(|V| V.as_str()).unwrap_or("").to_string();

	if Plaintext.is_empty() {
		return Ok(json!(""));
	}

	let KeyBytes = DeriveKey().map_err(|E| format!("encryption:encrypt unavailable - {E}"))?;

	let UnboundK = UnboundKey::new(&AES_256_GCM, &KeyBytes).map_err(|E| format!("encrypt key: {E:?}"))?;

	let Key = LessSafeKey::new(UnboundK);

	let Rng = SystemRandom::new();

	let mut NonceBytes = [0u8; 12];

	Rng.fill(&mut NonceBytes).map_err(|E| format!("encrypt rng: {E:?}"))?;

	let NonceVal = Nonce::assume_unique_for_key(NonceBytes);

	let mut Data = Plaintext.into_bytes();

	Key.seal_in_place_append_tag(NonceVal, Aad::empty(), &mut Data)
		.map_err(|E| format!("encrypt seal: {E:?}"))?;

	let mut Out = NonceBytes.to_vec();

	Out.extend_from_slice(&Data);

	dev_log!(
		"encryption",
		"encryption:encrypt {} bytes → {} bytes",
		Out.len() - 12 - 16,
		Out.len()
	);

	Ok(json!(B64.encode(&Out)))
}
