//! `TerminalState::GetAll`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{
		Arc,
		Mutex as StandardMutex,
		atomic::{AtomicU64, Ordering as AtomicOrdering},
	},
};
use crate::{ApplicationState::DTO::TerminalStateDTO::TerminalStateDTO, dev_log};

pub fn Fn(This:&Struct) -> HashMap<u64, TerminalStateDTO> {
		This.ActiveTerminals
			.lock()
			.ok()
			.map(|guard| {
				guard
					.iter()
					.filter_map(|(id, arc)| arc.lock().ok().map(|dto| (*id, dto.clone())))
					.collect()
			})
			.unwrap_or_default()
	}
