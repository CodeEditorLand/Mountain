//! `DiagnosticsState::ClearByOwnerAndResource`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO, dev_log};

pub fn Fn(This:&Struct, owner:&str, resource:&str) {
		if let Ok(mut guard) = This.DiagnosticsMap.lock() {
			if let Some(resources) = guard.get_mut(owner) {
				resources.remove(resource);

				dev_log!("extensions", "[DiagnosticsState] Diagnostics cleared for owner and resource");
			}
		}
	}
