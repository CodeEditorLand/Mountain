use std::sync::{Arc, Mutex as StandardMutex};

use log::debug;
use serde::{Deserialize, Serialize};

/// A single registered dynamic keybinding entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeybindingEntry {
	/// Command identifier (e.g. "workbench.action.files.save").
	pub CommandId:String,
	/// Key expression (e.g. "ctrl+s", "cmd+shift+p").
	pub Keybinding:String,
	/// Optional when-clause (e.g. "editorFocus && !editorReadonly").
	pub When:Option<String>,
}

/// Stores dynamically registered keyboard shortcuts.
#[derive(Clone)]
pub struct KeybindingState {
	Entries:Arc<StandardMutex<Vec<KeybindingEntry>>>,
}

impl Default for KeybindingState {
	fn default() -> Self {
		debug!("[KeybindingState] Initializing default keybinding state...");
		Self { Entries:Arc::new(StandardMutex::new(Vec::new())) }
	}
}

impl KeybindingState {
	/// Register a dynamic keybinding (replaces any existing entry for the same
	/// command).
	pub fn AddKeybinding(&self, CommandId:String, Keybinding:String, When:Option<String>) {
		if let Ok(mut Guard) = self.Entries.lock() {
			Guard.retain(|E| E.CommandId != CommandId);
			Guard.push(KeybindingEntry { CommandId:CommandId.clone(), Keybinding, When });
			debug!("[KeybindingState] Keybinding added for: {}", CommandId);
		}
	}

	/// Remove all dynamic keybindings for a command.
	pub fn RemoveKeybinding(&self, CommandId:&str) {
		if let Ok(mut Guard) = self.Entries.lock() {
			Guard.retain(|E| E.CommandId != CommandId);
			debug!("[KeybindingState] Keybinding removed for: {}", CommandId);
		}
	}

	/// Return the resolved keybinding string for a command, or `None`.
	pub fn LookupKeybinding(&self, CommandId:&str) -> Option<String> {
		self.Entries
			.lock()
			.ok()
			.and_then(|Guard| Guard.iter().find(|E| E.CommandId == CommandId).map(|E| E.Keybinding.clone()))
	}

	/// Return all registered dynamic keybinding entries.
	pub fn GetAllKeybindings(&self) -> Vec<KeybindingEntry> {
		self.Entries.lock().ok().map(|Guard| Guard.clone()).unwrap_or_default()
	}
}
