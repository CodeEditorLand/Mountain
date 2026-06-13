//! Wire method: `nativeHost:onDidChangeMaximizeState`.
//!
//! Bridge: Cocoon notifies Mountain when maximize state changes (e.g.
//! when the native window is maximized by the OS or Tauri). Forwards the
//! payload to Sky so Wind's listen() bridge receives window state events.

use serde_json::Value;
use tauri::{AppHandle, Emitter};

pub fn Fn(ApplicationHandle:&AppHandle, Arguments:&[Value]) -> Result<Value, String> {
	let Payload = Arguments.first().cloned().unwrap_or(Value::Null);

	let _ = ApplicationHandle.emit("sky://window/maximize-changed", &Payload);

	Ok(Value::Null)
}
