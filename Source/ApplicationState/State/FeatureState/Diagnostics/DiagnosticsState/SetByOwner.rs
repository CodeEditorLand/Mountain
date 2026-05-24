//! `DiagnosticsState::SetByOwner`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO, dev_log};

pub fn Fn(This:&Struct, owner:String, diagnostics:HashMap<String, Vec<MarkerDataDTO>>) {
		if let Ok(mut guard) = This.DiagnosticsMap.lock() {
			guard.insert(owner, diagnostics);

			dev_log!("extensions", "[DiagnosticsState] Diagnostics updated for owner");
		}
	}
