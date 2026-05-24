//! `DiagnosticsState::GetByOwnerAndResource`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO, dev_log};

pub fn Fn(This:&Struct, owner:&str, resource:&str) -> Vec<MarkerDataDTO> {
		This.DiagnosticsMap
			.lock()
			.ok()
			.and_then(|guard| guard.get(owner).and_then(|resources| resources.get(resource).cloned()))
			.unwrap_or_default()
	}
