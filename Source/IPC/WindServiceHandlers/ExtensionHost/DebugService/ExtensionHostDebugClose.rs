//! `DebugService::ExtensionHostDebugClose`

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

pub async fn Fn(ApplicationHandle:AppHandle) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionhostdebugservice:close");

	if let Err(Error) = ApplicationHandle.emit("sky://exthost/debug-close", json!({})) {
		crate::dev_log!("exthost", "warn: extensionhostdebugservice:close emit failed: {}", Error);
	}

	Ok(Value::Null)
}
