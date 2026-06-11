use std::sync::Arc;

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

use crate::dev_log;

/// A single registered dynamic keybinding entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeybindingEntry {
	/// Command identifier (e.g. "workbench.action.files.save").
	pub CommandId:String,

	/// Key expression (e.g. "ctrl+s", "cmd+shift+p").
	pub Keybinding:String,

	/// Optional when-clause (e.g. "editorFocus && !editorReadonly").
	pub When:Option<String>,

	/// Origin of the entry: extension identifier for
	/// `RegisterExtensionKeybindings`, `None` for `keybinding:add` entries
	/// registered without an owner.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub Source:Option<String>,
}

/// Stores dynamically registered keyboard shortcuts.
#[derive(Clone)]
pub struct KeybindingState {
	Entries:Arc<Mutex<Vec<KeybindingEntry>>>,
}

impl Default for KeybindingState {
	fn default() -> Self {
		dev_log!("keybinding", "[KeybindingState] Initializing default keybinding state...");

		Self { Entries:Arc::new(Mutex::new(Vec::new())) }
	}
}

impl KeybindingState {
	/// Register a dynamic keybinding (replaces any existing entry for the same
	/// command).
	pub fn AddKeybinding(&self, CommandId:String, Keybinding:String, When:Option<String>) {
		let mut Guard = self.Entries.lock();

		Guard.retain(|E| E.CommandId != CommandId);

		Guard.push(KeybindingEntry { CommandId:CommandId.clone(), Keybinding, When, Source:None });

		dev_log!("keybinding", "[KeybindingState] Keybinding added for: {}", CommandId);
	}

	/// Register a dynamic keybinding owned by a source (extension
	/// identifier). Unlike `AddKeybinding` this does NOT displace entries
	/// for the same command from other sources - an extension contributing
	/// a binding must not silently erase a user-registered one. It replaces
	/// only its own previous entry for the same command.
	pub fn AddKeybindingFromSource(&self, CommandId:String, Keybinding:String, When:Option<String>, Source:String) {
		let mut Guard = self.Entries.lock();

		Guard.retain(|E| !(E.CommandId == CommandId && E.Source.as_deref() == Some(Source.as_str())));

		Guard.push(KeybindingEntry {
			CommandId:CommandId.clone(),
			Keybinding,
			When,
			Source:Some(Source.clone()),
		});

		dev_log!("keybinding", "[KeybindingState] Keybinding added for: {} (source: {})", CommandId, Source);
	}

	/// Remove every dynamic keybinding registered by a source. Returns the
	/// number of entries removed so callers can report it.
	pub fn RemoveKeybindingsBySource(&self, Source:&str) -> usize {
		let mut Guard = self.Entries.lock();

		let Before = Guard.len();

		Guard.retain(|E| E.Source.as_deref() != Some(Source));

		let Removed = Before - Guard.len();

		dev_log!("keybinding", "[KeybindingState] {} keybinding(s) removed for source: {}", Removed, Source);

		Removed
	}

	/// Remove all dynamic keybindings for a command.
	pub fn RemoveKeybinding(&self, CommandId:&str) {
		let mut Guard = self.Entries.lock();

		Guard.retain(|E| E.CommandId != CommandId);

		dev_log!("keybinding", "[KeybindingState] Keybinding removed for: {}", CommandId);
	}

	/// Return the resolved keybinding string for a command, or `None`.
	pub fn LookupKeybinding(&self, CommandId:&str) -> Option<String> {
		self.Entries
			.lock()
			.iter()
			.find(|E| E.CommandId == CommandId)
			.map(|E| E.Keybinding.clone())
	}

	/// Return all registered dynamic keybinding entries.
	pub fn GetAllKeybindings(&self) -> Vec<KeybindingEntry> { self.Entries.lock().clone() }
}
