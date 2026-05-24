//! `ScannedExtensions::Remove`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

pub fn Fn(This:&Struct, identifier:&str) {
		if let Ok(mut guard) = This.ScannedExtensions.lock() {
			guard.remove(identifier);

			dev_log!("extensions", "[ScannedExtensions] Extension removed: {}", identifier);
		}
	}
