//! `ScannedExtensions::Clear`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

pub fn Fn(This:&Struct) {
		if let Ok(mut guard) = This.ScannedExtensions.lock() {
			guard.clear();

			dev_log!("extensions", "[ScannedExtensions] All extensions cleared");
		}
	}
