//! `MountainDiscoverServices` Tauri command - run a
//! one-shot discovery pass and return the populated
//! `ServiceInfo::Struct` list.

use tauri::Manager;

use crate::{
	IPC::StatusReporter::{Reporter::Struct as Reporter, ServiceInfo::Struct as ServiceInfo},
	dev_log,
};

#[tauri::command]
pub async fn Fn(app_handle:tauri::AppHandle) -> Result<Vec<ServiceInfo>, String> {
	dev_log!("lifecycle", "Tauri command: discover_services");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter.DiscoverServices().await
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
