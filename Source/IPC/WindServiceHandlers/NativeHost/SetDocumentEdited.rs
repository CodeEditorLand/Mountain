//! Wire method: `nativeHost:setDocumentEdited`.
//!
//! Tauri 2.x does not expose `NSWindow.setDocumentEdited` directly.
//! Prefixes the window title with '•' as a visual proxy for the macOS
//! dirty-dot indicator.

use serde_json::Value;
use tauri::{AppHandle, Manager};

pub fn Fn(ApplicationHandle:&AppHandle, Arguments:&[Value]) -> Result<Value, String> {
	let Edited = Arguments.first().and_then(Value::as_bool).unwrap_or(false);

	if let Some(Win) = ApplicationHandle.get_webview_window("main") {
		if let Ok(Current) = Win.title() {
			let New = if Edited {
				if Current.starts_with('•') { Current } else { format!("• {}", Current) }
			} else {
				Current.trim_start_matches('•').trim().to_string()
			};

			let _ = Win.set_title(&New);
		}
	}

	Ok(Value::Null)
}
