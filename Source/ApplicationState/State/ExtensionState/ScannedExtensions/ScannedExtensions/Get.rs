//! `ScannedExtensions::Get`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

pub fn Fn(This:&Struct, identifier:&str) -> Option<ExtensionDescriptionStateDTO> {
		This.ScannedExtensions
			.lock()
			.ok()
			.and_then(|guard| guard.get(identifier).cloned())
	}
