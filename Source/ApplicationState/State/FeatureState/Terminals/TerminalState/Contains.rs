//! `TerminalState::Contains`

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

pub fn Fn(This:&Struct, id:u64) -> bool {
		This.ActiveTerminals
			.lock()
			.ok()
			.map(|guard| guard.contains_key(&id))
			.unwrap_or(false)
	}
