//! `ScannedExtensions::GetAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

pub fn Fn(This:&Struct) -> HashMap<String, ExtensionDescriptionStateDTO> {
		This.ScannedExtensions
			.lock()
			.ok()
			.map(|guard| guard.clone())
			.unwrap_or_default()
	}
