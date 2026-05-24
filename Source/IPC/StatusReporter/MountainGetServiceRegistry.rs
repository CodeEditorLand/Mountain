//! `MountainGetServiceRegistry` Tauri command - returns
//! the full `ServiceRegistry::Struct` snapshot.

use tauri::Manager;

use crate::{
	IPC::StatusReporter::{Reporter::Struct as Reporter, ServiceRegistry::Struct as ServiceRegistry},
	dev_log,
};

#[tauri::command]
pub async fn Fn(app_handle:tauri::AppHandle) -> Result<ServiceRegistry, String> {
	dev_log!("lifecycle", "Tauri command: get_service_registry");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter.GetServiceRegistry().await
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
