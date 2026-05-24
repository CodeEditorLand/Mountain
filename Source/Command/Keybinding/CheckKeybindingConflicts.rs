//! Tauri command - detect chord-sequence overlaps in the current
//! keybinding registry. Stub returns no conflicts; pending real
//! implementation that scans the resolved set and reports source +
//! command for each clash.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Keybinding::KeybindingProvider::KeybindingProvider};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Wry, command};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

#[command]
pub async fn Fn(ApplicationHandle:AppHandle<Wry>, Keybinding:String) -> Result<Value, String> {
	dev_log!("keybinding", "checking conflicts for keybinding: {}", Keybinding);

	let RunTime = ApplicationHandle.state::<Arc<Runtime>>().inner().clone();

	let _Provider:Arc<dyn KeybindingProvider> = RunTime.Environment.Require();

	Ok(json!({ "conflicts": [] }))
}
