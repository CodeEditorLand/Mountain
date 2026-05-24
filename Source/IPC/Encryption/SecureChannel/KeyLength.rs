//! `SecureChannel::KeyLength`

use super::Struct;
use ring::{
	aead::{self, AES_256_GCM, LessSafeKey, UnboundKey},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct) -> usize {
		32 // AES-256 uses 32-byte keys
	}
