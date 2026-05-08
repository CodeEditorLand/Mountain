#![allow(non_snake_case)]

//! Tauri command - register keybindings contributed by an extension.
//! Stub returns success; pending real implementation that validates,
//! checks permissions, persists to ApplicationState, and updates the
//! resolution cache.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Wry, command};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

#[command]
pub async fn RegisterExtensionKeybindings(
	ApplicationHandle:AppHandle<Wry>,

	ExtensionIdentifier:String,

	_Keybindings:Value,
) -> Result<Value, String> {
	dev_log!("keybinding", "registering keybindings for extension: {}", ExtensionIdentifier);

	let RunTime = ApplicationHandle.state::<Arc<Runtime>>().inner().clone();

	let _Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	Ok(json!({ "success": true }))
}
