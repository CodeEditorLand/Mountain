#![allow(non_snake_case)]

//! `mountain_get_performance_stats` Tauri command - returns
//! the cumulative `PerformanceStats::Struct`.

use tauri::Manager;

use crate::{
	IPC::AdvancedFeatures::{Features::Struct as Features, PerformanceStats::Struct as PerformanceStats},
	dev_log,
};

#[tauri::command]
pub async fn mountain_get_performance_stats(app_handle:tauri::AppHandle) -> Result<PerformanceStats, String> {
	dev_log!("lifecycle", "Tauri command: get_performance_stats");

	if let Some(features) = app_handle.try_state::<Features>() {
		features.get_performance_stats().await
	} else {
		Err("AdvancedFeatures not found in application state".to_string())
	}
}
