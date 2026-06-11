//! Tauri command - return user-defined keybinding overrides, read from
//! `keybindings.json` in the app config directory (the same file
//! `KeybindingProvider::GetResolvedKeybinding` overlays last). Unbind
//! rules (`-command`) are returned as-is so the shortcuts UI can show
//! and edit them. A missing or malformed file yields an empty list.

use std::sync::Arc;

use CommonLibrary::FileSystem::ReadFile::ReadFile;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Wry, command};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

#[command]
pub async fn GetUserKeybindings(ApplicationHandle:AppHandle<Wry>) -> Result<Value, String> {
	dev_log!("keybinding", "getting user keybindings for UI");

	let RunTime = ApplicationHandle.state::<Arc<Runtime>>().inner().clone();

	let UserKeybindingsPath = ApplicationHandle
		.path()
		.app_config_dir()
		.map_err(|Error| format!("Cannot find app config dir: {}", Error))?
		.join("keybindings.json");

	let Keybindings = match RunTime.Run(ReadFile(UserKeybindingsPath)).await {
		Ok(Content) => {
			match serde_json::from_slice::<Value>(&Content) {
				Ok(Value::Array(Rules)) => Rules,
				Ok(_) => {
					dev_log!("keybinding", "warn: keybindings.json is not an array");

					Vec::new()
				},
				Err(Error) => {
					dev_log!("keybinding", "warn: keybindings.json is malformed: {}", Error);

					Vec::new()
				},
			}
		},
		// Absent file is the normal first-run state.
		Err(_) => Vec::new(),
	};

	Ok(json!({ "keybindings": Keybindings }))
}
