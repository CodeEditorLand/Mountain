//! Wire method: `extensionhostdebugservice:close`.
//! Emits `sky://exthost/debug-close` so the Sky bridge can react.

use serde_json::{Value, json};

use tauri::{AppHandle, Emitter};

pub async fn Fn(ApplicationHandle:AppHandle) -> Result<Value, String> {

	crate::dev_log!("exthost", "extensionhostdebugservice:close");

	if let Err(Error) = ApplicationHandle.emit("sky://exthost/debug-close", json!({})) {
		crate::dev_log!("exthost", "warn: extensionhostdebugservice:close emit failed: {}", Error);
	}

	Ok(Value::Null)
}
