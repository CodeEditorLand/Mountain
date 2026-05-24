//! `DiagnosticsState::GetAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MarkerDataDTO::MarkerDataDTO, dev_log};

pub fn Fn(This:&Struct) -> HashMap<String, HashMap<String, Vec<MarkerDataDTO>>> {
		This.DiagnosticsMap.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}
