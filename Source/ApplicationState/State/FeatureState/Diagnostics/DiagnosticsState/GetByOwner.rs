//! `DiagnosticsState::GetByOwner`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO, dev_log};

pub fn Fn(This:&Struct, owner:&str) -> HashMap<String, Vec<MarkerDataDTO>> {
		This.DiagnosticsMap
			.lock()
			.ok()
			.and_then(|guard| guard.get(owner).cloned())
			.unwrap_or_default()
	}
