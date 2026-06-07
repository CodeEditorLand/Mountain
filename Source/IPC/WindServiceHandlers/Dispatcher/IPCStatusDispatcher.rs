//! IPC Status command dispatcher.

use serde_json::{Value, json};

/// Dispatches IPC status commands.
///
/// Handled commands:
/// - `mountain_get_status`
/// - `mountain_get_configuration`
/// - `mountain_get_services_status`
/// - `mountain_get_state`
pub async fn dispatch_ipc_status(
	app_handle:&tauri::AppHandle,

	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,

	command:&str,
) -> Result<Value, String> {
	match command {
		"mountain_get_status" => {
			Ok(json!({
				"connected": true,
				"version": "1.0.0"
			}))
		},

		"mountain_get_configuration" => {
			let config = runtime.Environment.ApplicationState.Configuration.GetGlobalConfiguration();

			Ok(config)
		},

		"mountain_get_services_status" => {
			let cocoon_connected = crate::Vine::Client::IsClientConnected::Fn("cocoon-main");

			let active_document = runtime.Environment.ApplicationState.Workspace.GetActiveDocumentURI();

			Ok(json!({
				"cocoon": { "connected": cocoon_connected },
				"vine": { "running": true }
			}))
		},

		"mountain_get_state" => {
			let folder_count = runtime.Environment.ApplicationState.Workspace.WorkspaceFolders.lock().len();

			Ok(json!({
				"workspace": { "folderCount": folder_count },
				"activeDocument": runtime.Environment.ApplicationState.Workspace.GetActiveDocumentURI()
			}))
		},

		_ => Err(format!("Unknown IPC status command: {}", command)),
	}
}
