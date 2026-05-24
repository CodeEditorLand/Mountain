//! `KeybindingState::AddKeybinding`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use serde::{Deserialize, Serialize};
use crate::dev_log;

pub fn Fn(This:&Struct, CommandId:String, Keybinding:String, When:Option<String>) {
		if let Ok(mut Guard) = This.Entries.lock() {
			Guard.retain(|E| E.CommandId != CommandId);

			Guard.push(KeybindingEntry { CommandId:CommandId.clone(), Keybinding, When });

			dev_log!("keybinding", "[KeybindingState] Keybinding added for: {}", CommandId);
		}
	}
