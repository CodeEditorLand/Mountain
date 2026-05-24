//! `DocumentState::Clear`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO, dev_log};

pub fn Fn(This:&Struct) {
		if let Ok(mut guard) = This.OpenDocuments.lock() {
			guard.clear();

			dev_log!("model", "[DocumentState] All documents cleared");
		}
	}
