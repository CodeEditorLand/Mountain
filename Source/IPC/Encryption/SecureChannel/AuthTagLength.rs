//! `SecureChannel::AuthTagLength`

use super::Struct;
use ring::{
	aead::{self, AES_256_GCM, LessSafeKey, UnboundKey},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct) -> usize { AES_256_GCM.tag_len() }
