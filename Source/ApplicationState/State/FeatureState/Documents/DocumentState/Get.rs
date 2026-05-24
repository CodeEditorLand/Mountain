//! `DocumentState::Get`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO, dev_log};

pub fn Fn(This:&Struct, uri:&str) -> Option<DocumentStateDTO> {
		This.OpenDocuments.lock().ok().and_then(|guard| guard.get(uri).cloned())
	}
