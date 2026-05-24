//! `TerminalState::GetArc`

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

pub fn Fn(This:&Struct, id:u64) -> Option<Arc<StandardMutex<TerminalStateDTO>>> {
		This.ActiveTerminals.lock().ok().and_then(|guard| guard.get(&id).cloned())
	}
