#![allow(non_snake_case)]

//! `mountain_start_service_discovery` Tauri command - kicks
//! off the periodic service-discovery background task driven
//! by `ServiceRegistry::Struct::discovery_interval`.

use tauri::Manager;

use crate::{IPC::StatusReporter::Reporter::Struct as Reporter, dev_log};

#[tauri::command]
pub async fn mountain_start_service_discovery(app_handle:tauri::AppHandle) -> Result<(), String> {
	dev_log!("lifecycle", "Tauri command: start_service_discovery");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter.start_periodic_discovery().await
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
