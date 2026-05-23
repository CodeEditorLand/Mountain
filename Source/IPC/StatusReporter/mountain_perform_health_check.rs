
//! `mountain_perform_health_check` Tauri command - runs a
//! synchronous health-check pass and returns the resulting
//! `HealthMonitor::Struct`.

use tauri::Manager;

use crate::{
	IPC::StatusReporter::{HealthMonitor::Struct as HealthMonitor, Reporter::Struct as Reporter},
	dev_log,
};

#[tauri::command]
pub async fn mountain_perform_health_check(app_handle:tauri::AppHandle) -> Result<HealthMonitor, String> {
	dev_log!("lifecycle", "Tauri command: perform_health_check");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter.perform_health_check().await?;

		reporter.get_health_status()
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
