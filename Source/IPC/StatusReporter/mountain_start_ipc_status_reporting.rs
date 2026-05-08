#![allow(non_snake_case)]

//! `mountain_start_ipc_status_reporting` Tauri command - kick
//! off the periodic Sky-emit loop with `interval_seconds`
//! between snapshots.

use tauri::Manager;

use crate::{IPC::StatusReporter::Reporter::Struct as Reporter, dev_log};

#[tauri::command]
pub async fn mountain_start_ipc_status_reporting(
	app_handle:tauri::AppHandle,

	interval_seconds:u64,
) -> Result<serde_json::Value, String> {
	dev_log!("lifecycle", "Tauri command: start_ipc_status_reporting");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter
			.start_periodic_reporting(interval_seconds)
			.await
			.map(|_| serde_json::json!({ "status": "started", "interval_seconds": interval_seconds }))
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
