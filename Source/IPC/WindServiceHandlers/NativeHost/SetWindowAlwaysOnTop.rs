//! Wire method: `nativeHost:setWindowAlwaysOnTop`.
//!
//! Sets the always-on-top flag on the main window based on the first argument.

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_bool, dev_log};

pub fn Fn(ApplicationHandle:&AppHandle, command:&str, Arguments:&[Value]) -> Result<Value, String> {
	dev_log!("window", "{}", command);

	let OnTop = arg_bool(Arguments, 0);

	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		let _ = Window.set_always_on_top(OnTop);
	}

	Ok(Value::Null)
}
