//! `nativeHost:openDevTools` - open the WebKit inspector for the main window.
//! Requires the `devtools` Tauri feature (already enabled in the debug profile
//! via `TAURI_DEV_TOOLS` env or cargo feature flag).

use serde_json::Value;

use tauri::{AppHandle, Manager};

use crate::dev_log;

pub async fn Fn(ApplicationHandle:AppHandle, _Arguments:Vec<Value>) -> Result<Value, String> {

	dev_log!("devtools", "nativeHost:openDevTools");

	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		Window.open_devtools();
	}

	Ok(Value::Null)
}
