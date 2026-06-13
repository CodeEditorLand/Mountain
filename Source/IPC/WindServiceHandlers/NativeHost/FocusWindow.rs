//! Wire method: `nativeHost:focusWindow`.
//!
//! Sets focus on the main Tauri webview window.

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::dev_log;

pub fn Fn(ApplicationHandle:&AppHandle, command:&str) -> Result<Value, String> {
	dev_log!("window", "{}", command);

	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		let _ = Window.set_focus();
	}

	Ok(Value::Null)
}
