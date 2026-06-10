//! Tauri command - remove keybindings registered by a given extension.
//! Stub returns success; pending real implementation that filters by
//! source identifier and clears the affected resolution cache.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};
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

	let _Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	Ok(json!({ "success": true }))
}
