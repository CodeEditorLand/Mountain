//! `DiagnosticsState::SetByOwnerAndResource`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO, dev_log};

pub fn Fn(This:&Struct, owner:String, resource:String, markers:Vec<MarkerDataDTO>) {
		if let Ok(mut guard) = This.DiagnosticsMap.lock() {
			guard.entry(owner).or_insert_with(HashMap::new).insert(resource, markers);

			dev_log!("extensions", "[DiagnosticsState] Diagnostics updated for owner and resource");
		}
	}
