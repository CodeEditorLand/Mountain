//! `MountainGetPerformanceMetrics` Tauri command - returns
//! the latest cached `PerformanceMetrics::Struct` snapshot.

use tauri::Manager;

use crate::{
	IPC::StatusReporter::{PerformanceMetrics::Struct as PerformanceMetrics, Reporter::Struct as Reporter},
	dev_log,
};

#[tauri::command]
pub async fn Fn(app_handle:tauri::AppHandle) -> Result<PerformanceMetrics, String> {
	dev_log!("lifecycle", "Tauri command: get_performance_metrics");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter.GetPerformanceMetrics()
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
