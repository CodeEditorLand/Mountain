//! `nativeHost:reload` - reload the webview without restarting the process.
//! VS Code calls this from `ILifecycleMainService.reload()` for "Reload
//! Window" (Developer menu / Cmd+Shift+P → Reload Window).
//!
//! Before triggering the renderer reload we ask Cocoon to serialize every
//! webview panel that has a registered serializer via
//! `vscode.window.registerWebviewPanelSerializer`. The returned
//! `(viewType, state)` pairs land in the global memento under
//! `__webview_panel_state__`, where Sky's webview bridge picks them up
//! after the reload completes and asks Cocoon to deserialize each entry.
//!
//! Errors / timeouts on the serialize path are intentionally swallowed:
//! reload must remain instant for the operator; a missing panel state
//! is a survivable degradation, a frozen reload button is not.

use std::{sync::Arc, time::Duration};

use CommonLibrary::Storage::StorageProvider::StorageProvider;
use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

const PANEL_STATE_KEY:&str = "__webview_panel_state__";

pub async fn Fn(ApplicationHandle:AppHandle, _Arguments:Vec<Value>) -> Result<Value, String> {
	dev_log!("lifecycle", "nativeHost:reload - reloading webview");

	// Best-effort webview panel snapshot before the renderer reload wipes
	// in-memory state. 1.5s budget: a slow serializer can hold up the
	// reload, but only briefly - past the budget we proceed without state.
	if ::Vine::Client::IsClientConnected::Fn("cocoon-main") {
		let SerializeMethod = "ExtHostWebviewPanels$serializeAllWebviewPanels".to_string();

		let SerializeCall =
			::Vine::Client::SendRequest::Fn("cocoon-main", SerializeMethod, Value::Array(Vec::new()), 1500);

		match tokio::time::timeout(Duration::from_millis(1700), SerializeCall).await {
			Ok(Ok(Snapshot)) if !Snapshot.is_null() => {
				if let Some(Runtime) = ApplicationHandle.try_state::<Arc<ApplicationRunTime>>() {
					if let Err(StoreError) = Runtime
						.inner()
						.Environment
						.UpdateStorageValue(true, PANEL_STATE_KEY.to_string(), Some(Snapshot))
						.await
					{
						dev_log!(
							"lifecycle",
							"warn: [Reload] Failed to persist webview panel snapshot: {:?}",
							StoreError
						);
					}
				}
			},

			Ok(Ok(_)) => {
				// Empty / null snapshot - no panels needed serialization.
			},

			Ok(Err(GrpcError)) => {
				dev_log!("lifecycle", "warn: [Reload] serializeAllWebviewPanels failed: {:?}", GrpcError);
			},

			Err(_) => {
				dev_log!(
					"lifecycle",
					"warn: [Reload] serializeAllWebviewPanels timed out - proceeding without panel state"
				);
			},
		}
	}

	if let Some(Window) = ApplicationHandle.get_webview_window("main") {
		Window
			.eval("location.reload()")
			.map_err(|E| format!("reload eval failed: {E}"))?;
	}

	Ok(Value::Null)
}
