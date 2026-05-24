//! `TerminalState::GetNextTerminalIdentifier`

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

pub fn Fn(This:&Struct) -> u64 { This.NextTerminalIdentifier.fetch_add(1, AtomicOrdering::Relaxed) }
