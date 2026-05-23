//! `mountain_get_ipc_status` Tauri command - one-shot status
//! report (basic IPC slice only).

use tauri::Manager;

use crate::{IPC::StatusReporter::Reporter::Struct as Reporter, dev_log};

#[tauri::command]
pub async fn mountain_get_ipc_status(app_handle:tauri::AppHandle) -> Result<serde_json::Value, String> {
	dev_log!("lifecycle", "Tauri command: get_ipc_status");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter
			.generate_status_report()
			.await
			.map(|report| serde_json::to_value(report).unwrap_or(serde_json::Value::Null))
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
