#![allow(non_snake_case)]

//! `mountain_attempt_recovery` Tauri command - dispose +
//! reinitialise the IPC server and zero the error counter.

use tauri::Manager;

use crate::{IPC::StatusReporter::Reporter::Struct as Reporter, dev_log};

#[tauri::command]
pub async fn mountain_attempt_recovery(app_handle:tauri::AppHandle) -> Result<(), String> {
	dev_log!("lifecycle", "Tauri command: attempt_recovery");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter.attempt_recovery().await
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
