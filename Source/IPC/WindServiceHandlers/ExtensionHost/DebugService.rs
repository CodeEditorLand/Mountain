//! Wire methods: `extensionhostdebugservice:*`.
//! Bridges VS Code's `IExtensionHostDebugService` channel. `reload` triggers
//! a real Cocoon restart by emitting `sky://exthost/debug-reload` so Wind can
//! tear down caches before the fresh spawn. Other methods are acknowledged.

use serde_json::{Value, json};
use tauri::{AppHandle, Emitter};
use CommonLibrary::IPC::SkyEvent::SkyEvent;

pub async fn ExtensionHostDebugReload(ApplicationHandle:AppHandle) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionhostdebugservice:reload");

	if let Err(Error) = ApplicationHandle.emit(SkyEvent::ExtHostDebugReload.AsStr(), json!({})) {
		crate::dev_log!("exthost", "warn: extensionhostdebugservice:reload emit failed: {}", Error);
	}

	let _ =
		::Vine::Client::SendNotification::Fn("cocoon-main".to_string(), "extensionHost.reload".to_string(), json!({}))
			.await;

	Ok(Value::Null)
}

pub async fn ExtensionHostDebugClose(ApplicationHandle:AppHandle) -> Result<Value, String> {
	crate::dev_log!("exthost", "extensionhostdebugservice:close");

	if let Err(Error) = ApplicationHandle.emit("sky://exthost/debug-close", json!({})) {
		crate::dev_log!("exthost", "warn: extensionhostdebugservice:close emit failed: {}", Error);
	}

	let _ =
		::Vine::Client::SendNotification::Fn("cocoon-main".to_string(), "extensionHost.close".to_string(), json!({}))
			.await;

	Ok(Value::Null)
}
