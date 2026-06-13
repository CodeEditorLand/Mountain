//! Wire method: `nativeHost:setTitle` / `window:setTitle`.
//!
//! Sets the main window title from the first argument.

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_string, dev_log};

pub fn Fn(ApplicationHandle:&AppHandle, command:&str, Arguments:&[Value]) -> Result<Value, String> {
	dev_log!("window", "{}", command);

	let Title = arg_string(Arguments, 0);

	if !Title.is_empty() {
		if let Some(Win) = ApplicationHandle.get_webview_window("main") {
			let _ = Win.set_title(&Title);
		}
	}

	Ok(Value::Null)
}
