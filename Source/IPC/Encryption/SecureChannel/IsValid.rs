//! `SecureChannel::IsValid`

use super::Struct;
use ring::{
	aead::{self, AES_256_GCM, LessSafeKey, UnboundKey},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct) -> bool {
		This.nonce.len() == 12 // AES-256-GCM requires 12-byte nonce
			&& !This.ciphertext.is_empty()

			&& !This.hmac_tag.is_empty()
	}
