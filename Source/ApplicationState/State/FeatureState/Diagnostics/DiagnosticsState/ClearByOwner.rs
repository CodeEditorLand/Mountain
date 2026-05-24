//! `DiagnosticsState::ClearByOwner`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO, dev_log};

pub fn Fn(This:&Struct, owner:&str) {
		if let Ok(mut guard) = This.DiagnosticsMap.lock() {
			guard.remove(owner);

			dev_log!("extensions", "[DiagnosticsState] Diagnostics cleared for owner: {}", owner);
		}
	}
