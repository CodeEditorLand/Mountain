//! Wire method: `nativeHost:unmaximizeWindow`.
//!
//! Unmaximizes the main window and emits `sky://window/maximize-changed`
//! so Wind's listen() bridge receives window state events.

use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use crate::dev_log;

pub fn Fn(ApplicationHandle:&AppHandle, command:&str) -> Result<Value, String> {
	dev_log!("window", "{}", command);

	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		let _ = Window.unmaximize();
	}

	let _ = ApplicationHandle.emit("sky://window/maximize-changed", false);

	Ok(Value::Null)
}
