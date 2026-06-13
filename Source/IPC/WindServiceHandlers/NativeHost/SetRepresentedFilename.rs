//! Wire method: `nativeHost:setRepresentedFilename`.
//!
//! Sets the proxy icon in the macOS title bar. Tauri doesn't expose
//! `NSWindow.representedFilename` directly, so this sets the window
//! title to the filename component as a best-effort visual proxy.

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{IPC::WindServiceHandlers::Utilities::JsonValueHelpers::arg_string, dev_log};

pub fn Fn(ApplicationHandle:&AppHandle, command:&str, Arguments:&[Value]) -> Result<Value, String> {
	dev_log!("window", "{}", command);

	let Path = arg_string(Arguments, 0);

	if !Path.is_empty() {
		if let Some(Window) = ApplicationHandle.get_webview_window("main") {
			let Filename = std::path::Path::new(&Path)
				.file_name()
				.and_then(|N| N.to_str())
				.unwrap_or(&Path);

			let _ = Window.set_title(Filename);
		}
	}

	Ok(Value::Null)
}
