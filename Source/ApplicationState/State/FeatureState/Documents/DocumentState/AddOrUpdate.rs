//! `DocumentState::AddOrUpdate`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO, dev_log};

pub fn Fn(This:&Struct, uri:String, document:DocumentStateDTO) {
		if let Ok(mut guard) = This.OpenDocuments.lock() {
			guard.insert(uri, document);

			dev_log!("model", "[DocumentState] Document added/updated");
		}
	}
