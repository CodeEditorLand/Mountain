//! Wire method: `extensionhostdebugservice:reload`.
//! Emits `sky://exthost/debug-reload` so Wind can tear down caches before
//! a fresh Cocoon spawn, then notifies Cocoon so it can reinitialize.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

pub async fn Fn(ApplicationHandle:AppHandle) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionhostdebugservice:reload");

	if let Err(Error) = ApplicationHandle.emit(SkyEvent::ExtHostDebugReload.AsStr(), json!({})) {
		crate::dev_log!("exthost", "warn: extensionhostdebugservice:reload emit failed: {}", Error);
	}

	let _ = crate::Vine::Client::SendNotification::Fn(
		"cocoon-main".to_string(),
		"extensionHost.reload".to_string(),
		json!({}),
	)
	.await;

	Ok(Value::Null)
}
