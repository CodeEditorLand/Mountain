//! `Encrypt::GetKeyIdentifier`

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

pub fn Fn(This:&Struct) -> String {

		// Create a simple identifier from HMAC key (not the key itself)
		use ring::digest;

		let digest = digest::digest(&digest::SHA256, &This.hmac_key);

		general_purpose::STANDARD.encode(digest.as_ref())[..32].to_string()
	}
