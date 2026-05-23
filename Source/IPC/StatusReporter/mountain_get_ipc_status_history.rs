
//! `mountain_get_ipc_status_history` Tauri command - returns
//! the last-100 ring buffer of `IPCStatusReport::Struct`.

use tauri::Manager;

use crate::{IPC::StatusReporter::Reporter::Struct as Reporter, dev_log};

#[tauri::command]
pub async fn mountain_get_ipc_status_history(app_handle:tauri::AppHandle) -> Result<serde_json::Value, String> {
	dev_log!("lifecycle", "Tauri command: get_ipc_status_history");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter
			.get_status_history()
			.map(|history| serde_json::to_value(history).unwrap_or(serde_json::Value::Null))
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
