//! `ScannedExtensions::AddOrUpdate`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

pub fn Fn(This:&Struct, identifier:String, extension:ExtensionDescriptionStateDTO) {
		if let Ok(mut guard) = This.ScannedExtensions.lock() {
			guard.insert(identifier, extension);

			dev_log!("extensions", "[ScannedExtensions] Extension added/updated");
		}
	}
