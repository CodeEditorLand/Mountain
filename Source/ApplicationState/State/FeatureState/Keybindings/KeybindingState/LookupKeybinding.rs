//! `KeybindingState::LookupKeybinding`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use serde::{Deserialize, Serialize};
use crate::dev_log;

pub fn Fn(This:&Struct, CommandId:&str) -> Option<String> {
		This.Entries
			.lock()
			.ok()
			.and_then(|Guard| Guard.iter().find(|E| E.CommandId == CommandId).map(|E| E.Keybinding.clone()))
	}
