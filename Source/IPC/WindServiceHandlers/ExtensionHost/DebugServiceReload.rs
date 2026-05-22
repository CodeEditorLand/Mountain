#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `extensionhostdebugservice:reload`.
//! Emits `sky://exthost/debug-reload` so Wind can tear down caches before
//! a fresh Cocoon spawn.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

pub async fn Fn(ApplicationHandle:AppHandle) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionhostdebugservice:reload");

	if let Err(Error) = ApplicationHandle.emit(SkyEvent::ExtHostDebugReload.AsStr(), json!({})) {
		crate::dev_log!("exthost", "warn: extensionhostdebugservice:reload emit failed: {}", Error);
	}

	Ok(Value::Null)
}
