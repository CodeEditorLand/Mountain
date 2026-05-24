//! `TerminalState::AddOrUpdate`

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

pub fn Fn(This:&Struct, id:u64, terminal:TerminalStateDTO) {
		if let Ok(mut guard) = This.ActiveTerminals.lock() {
			guard.insert(id, Arc::new(StandardMutex::new(terminal)));

			dev_log!("terminal", "[TerminalState] Terminal added/updated with ID: {}", id);
		}
	}
