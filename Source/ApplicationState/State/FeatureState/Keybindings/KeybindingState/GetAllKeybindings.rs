//! `KeybindingState::GetAllKeybindings`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use serde::{Deserialize, Serialize};
use crate::dev_log;

pub fn Fn(This:&Struct) -> Vec<KeybindingEntry> {
		This.Entries.lock().ok().map(|Guard| Guard.clone()).unwrap_or_default()
	}
