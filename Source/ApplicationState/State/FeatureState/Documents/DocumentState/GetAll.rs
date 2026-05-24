//! `DocumentState::GetAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::DocumentStateDTO::DocumentStateDTO, dev_log};

pub fn Fn(This:&Struct) -> HashMap<String, DocumentStateDTO> {
		This.OpenDocuments.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}
