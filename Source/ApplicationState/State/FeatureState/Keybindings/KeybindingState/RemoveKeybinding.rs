//! `KeybindingState::RemoveKeybinding`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use serde::{Deserialize, Serialize};
use crate::dev_log;

pub fn Fn(This:&Struct, CommandId:&str) {
		if let Ok(mut Guard) = This.Entries.lock() {
			Guard.retain(|E| E.CommandId != CommandId);

			dev_log!("keybinding", "[KeybindingState] Keybinding removed for: {}", CommandId);
		}
	}
