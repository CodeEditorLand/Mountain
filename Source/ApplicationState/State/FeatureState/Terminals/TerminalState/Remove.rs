//! `TerminalState::Remove`

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

pub fn Fn(This:&Struct, id:u64) {
		if let Ok(mut guard) = This.ActiveTerminals.lock() {
			guard.remove(&id);

			dev_log!("terminal", "[TerminalState] Terminal removed with ID: {}", id);
		}
	}
