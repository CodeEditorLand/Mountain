#![allow(non_snake_case)]

//! Generic encrypted-message wrapper carrying additional
//! routing headers and a protocol version. The phantom `T`
//! is the original plaintext type; the wrapper itself
//! serialises only the encrypted envelope + headers + version.

use std::{collections::HashMap, marker::PhantomData};

use serde::{Deserialize, Serialize};

use crate::IPC::Enhanced::SecureMessageChannel::EncryptedMessage;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct<T> {
	pub encrypted:EncryptedMessage::Struct,

	pub headers:HashMap<String, String>,

	pub version:String,

	#[serde(skip)]
	pub(super) _marker:PhantomData<T>,
}
