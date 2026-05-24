//! `MountainStartIpcStatusReporting` Tauri command - kick
//! off the periodic Sky-emit loop with `interval_seconds`
//! between snapshots.

use tauri::Manager;

use crate::{IPC::StatusReporter::Reporter::Struct as Reporter, dev_log};

#[tauri::command]
pub async fn Fn(
	app_handle:tauri::AppHandle,

	interval_seconds:u64,
) -> Result<serde_json::Value, String> {
	dev_log!("lifecycle", "Tauri command: start_ipc_status_reporting");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter
			.StartPeriodicReporting(interval_seconds)
			.await
			.map(|_| serde_json::json!({ "status": "started", "interval_seconds": interval_seconds }))
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
