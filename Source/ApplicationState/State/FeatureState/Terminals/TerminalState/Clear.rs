//! `TerminalState::Clear`

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

pub fn Fn(This:&Struct) {
		if let Ok(mut guard) = This.ActiveTerminals.lock() {
			guard.clear();

			dev_log!("terminal", "[TerminalState] All terminals cleared");
		}
	}
