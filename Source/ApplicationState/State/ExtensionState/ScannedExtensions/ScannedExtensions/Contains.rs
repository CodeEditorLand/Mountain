//! `ScannedExtensions::Contains`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

pub fn Fn(This:&Struct, identifier:&str) -> bool {
		This.ScannedExtensions
			.lock()
			.ok()
			.map(|guard| guard.contains_key(identifier))
			.unwrap_or(false)
	}
