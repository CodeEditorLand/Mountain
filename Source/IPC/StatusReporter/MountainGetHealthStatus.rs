//! `MountainGetHealthStatus` Tauri command - returns the
//! current `HealthMonitor::Struct` (score + active issues).

use tauri::Manager;

use crate::{
	IPC::StatusReporter::{HealthMonitor::Struct as HealthMonitor, Reporter::Struct as Reporter},
	dev_log,
};

#[tauri::command]
pub async fn Fn(app_handle:tauri::AppHandle) -> Result<HealthMonitor, String> {
	dev_log!("lifecycle", "Tauri command: get_health_status");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter.GetHealthStatus()
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
