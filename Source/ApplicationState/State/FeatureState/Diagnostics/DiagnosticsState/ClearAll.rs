//! `DiagnosticsState::ClearAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO, dev_log};

pub fn Fn(This:&Struct) {
		if let Ok(mut guard) = This.DiagnosticsMap.lock() {
			guard.clear();

			dev_log!("extensions", "[DiagnosticsState] All diagnostics cleared");
		}
	}
