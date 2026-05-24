//! `ScannedExtensions::SetAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, dev_log};

pub fn Fn(This:&Struct, extensions:HashMap<String, ExtensionDescriptionStateDTO>) {
		if let Ok(mut guard) = This.ScannedExtensions.lock() {
			*guard = extensions;
			dev_log!(
				"extensions",
				"[ScannedExtensions] Scanned extensions updated ({} extensions)",
				guard.len()
			);
		}
	}
