//! Wire method: `nativeHost:toggleFullScreen`.
//!
//! Toggles fullscreen mode on the main Tauri webview window.

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::dev_log;

pub fn Fn(ApplicationHandle:&AppHandle, command:&str) -> Result<Value, String> {
	dev_log!("window", "{}", command);

	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		let IsFullscreen = Window.is_fullscreen().unwrap_or(false);

		let _ = Window.set_fullscreen(!IsFullscreen);
	}

	Ok(Value::Null)
}
