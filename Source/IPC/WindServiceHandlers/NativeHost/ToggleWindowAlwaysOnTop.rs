//! Wire method: `nativeHost:toggleWindowAlwaysOnTop`.
//!
//! Toggles the always-on-top flag on the main window using a static
//! atomic for state tracking when no Cocoon round-trip is available.

use std::sync::atomic::{AtomicBool, Ordering};

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::dev_log;

pub fn Fn(ApplicationHandle:&AppHandle, command:&str) -> Result<Value, String> {
	dev_log!("window", "{}", command);

	static ALWAYS_ON_TOP:AtomicBool = AtomicBool::new(false);

	let Next = !ALWAYS_ON_TOP.fetch_xor(true, Ordering::Relaxed);

	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		let _ = Window.set_always_on_top(Next);
	}

	Ok(Value::Null)
}
