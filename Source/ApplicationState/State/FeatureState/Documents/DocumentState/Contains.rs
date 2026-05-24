//! `DocumentState::Contains`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO, dev_log};

pub fn Fn(This:&Struct, uri:&str) -> bool {
		This.OpenDocuments
			.lock()
			.ok()
			.map(|guard| guard.contains_key(uri))
			.unwrap_or(false)
	}
