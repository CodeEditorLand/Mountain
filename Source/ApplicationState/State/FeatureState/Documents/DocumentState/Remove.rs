//! `DocumentState::Remove`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO, dev_log};

pub fn Fn(This:&Struct, uri:&str) {
		if let Ok(mut guard) = This.OpenDocuments.lock() {
			guard.remove(uri);

			dev_log!("model", "[DocumentState] Document removed: {}", uri);
		}
	}
