//! Wire method: `extensionhostdebugservice:close`.
//! Emits `sky://exthost/debug-close` so the Sky bridge can react,
//! then notifies Cocoon so it can tear down its extension host state.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};

pub async fn Fn(ApplicationHandle:AppHandle) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionhostdebugservice:close");

	if let Err(Error) = ApplicationHandle.emit("sky://exthost/debug-close", json!({})) {
		crate::dev_log!("exthost", "warn: extensionhostdebugservice:close emit failed: {}", Error);
	}

	let _ =
		crate::Vine::Client::SendNotification::Fn("cocoon-main".to_string(), "extensionHost.close".to_string(), json!({}))
			.await;

	Ok(Value::Null)
}
