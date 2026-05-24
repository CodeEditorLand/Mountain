//! `DocumentState::GetURIs`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO, dev_log};

pub fn Fn(This:&Struct) -> Vec<String> {
		This.OpenDocuments
			.lock()
			.ok()
			.map(|guard| guard.keys().cloned().collect())
			.unwrap_or_default()
	}
