pub mod AddKeybinding;
pub mod RemoveKeybinding;
pub mod LookupKeybinding;
pub mod GetAllKeybindings;

use std::sync::{Arc, Mutex as StandardMutex};
use serde::{Deserialize, Serialize};
use crate::dev_log;

/// A single registered dynamic keybinding entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KeybindingEntry {
	/// Command identifier (e.g. "workbench.Action.files.save").
	pub CommandId:String,

	/// Key expression (e.g. "ctrl+s", "cmd+shift+p").
	pub Keybinding:String,

	/// Optional when-clause (e.g. "editorFocus && !editorReadonly").
	pub When:Option<String>,
}

/// Stores dynamically registered keyboard shortcuts.
#[derive(Clone)]
pub struct Struct {
	Entries:Arc<StandardMutex<Vec<KeybindingEntry>>>,
}

