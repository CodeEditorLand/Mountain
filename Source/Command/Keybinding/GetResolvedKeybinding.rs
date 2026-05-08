#![allow(non_snake_case)]

//! Tauri command - fetch the final resolved keybindings (default +
//! extension contributions + user overrides, weighted) for the keyboard
//! shortcuts UI.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};
use serde_json::Value;
use tauri::{AppHandle, Manager, Wry, command};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

#[command]
pub async fn GetResolvedKeybinding(ApplicationHandle:AppHandle<Wry>) -> Result<Value, String> {
	dev_log!("keybinding", "getting resolved keybindings for UI");

	let RunTime = ApplicationHandle.state::<Arc<Runtime>>().inner().clone();

	let Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	Provider.GetResolvedKeybinding().await.map_err(|Error| Error.to_string())
}
