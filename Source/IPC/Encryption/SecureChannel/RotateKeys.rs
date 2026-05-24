//! `SecureChannel::RotateKeys`

use super::Struct;
use ring::{
	aead::{self, AES_256_GCM, LessSafeKey, UnboundKey},
	hmac,
	rand::{SecureRandom, SystemRandom},
};
use serde::{Deserialize, Serialize};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&mut Struct) -> Result<(), String> {
		dev_log!("encryption", "[SecureMessageChannel] Rotating encryption keys");

		*self = Struct::new()?;

		dev_log!("encryption", "[SecureMessageChannel] Keys rotated successfully");

		Ok(())
	}
