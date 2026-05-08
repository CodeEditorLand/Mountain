#![allow(non_snake_case)]

//! `mountain_get_service_info` Tauri command - look up one
//! service by name in the registry.

use tauri::Manager;

use crate::{
	IPC::StatusReporter::{Reporter::Struct as Reporter, ServiceInfo::Struct as ServiceInfo},
	dev_log,
};

#[tauri::command]
pub async fn mountain_get_service_info(
	app_handle:tauri::AppHandle,

	service_name:String,
) -> Result<Option<ServiceInfo>, String> {
	dev_log!("lifecycle", "Tauri command: get_service_info");

	if let Some(reporter) = app_handle.try_state::<Reporter>() {
		reporter.get_service_info(&service_name).await
	} else {
		Err("StatusReporter not found in application state".to_string())
	}
}
