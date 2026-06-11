//! Tauri command - remove every dynamic keybinding registered by a given
//! extension (the `Source` tag written by `RegisterExtensionKeybindings`).
//! Returns the number of entries removed.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Wry, command};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

#[command]
pub async fn UnregisterExtensionKeybindings(
	ApplicationHandle:AppHandle<Wry>,

	ExtensionIdentifier:String,
) -> Result<Value, String> {
	dev_log!("keybinding", "unregistering keybindings for extension: {}", ExtensionIdentifier);

	let RunTime = ApplicationHandle.state::<Arc<Runtime>>().inner().clone();

	let Removed = RunTime
		.Environment
		.ApplicationState
		.Feature
		.Keybindings
		.RemoveKeybindingsBySource(&ExtensionIdentifier);

	Ok(json!({ "success": true, "removed": Removed }))
}
