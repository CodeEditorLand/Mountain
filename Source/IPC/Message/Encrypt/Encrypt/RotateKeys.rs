//! `Encrypt::RotateKeys`

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

pub fn Fn(This:&mut Struct) -> Result<(), String> {

		*self = Struct::new()?;
		Ok(())
	}
