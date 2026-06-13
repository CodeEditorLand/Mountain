//! Wire method: `nativeHost:closeWindow`.
//!
//! Destroys the main window. Uses `destroy()` instead of `close()` so the
//! `prevent_close` intercept registered in AppLifecycle is not re-entered.

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::dev_log;

pub fn Fn(ApplicationHandle:&AppHandle, command:&str) -> Result<Value, String> {
	dev_log!("window", "{}", command);

	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		let _ = Window.destroy();
	}

	Ok(Value::Null)
}
