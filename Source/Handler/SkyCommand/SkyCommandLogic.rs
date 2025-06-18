// @module SkyCommandLogic
// @description Contains the logic for Tauri commands invoked directly by the
// Sky frontend for window management and system information queries.

use std::collections::HashMap;

use log::debug;
use serde::Deserialize;
use sysinfo::{ProcessExt, ProcessRefreshKind, System, SystemExt};
use tauri::{Window, Wry, command};

use crate::Handler::error_utils;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct SetZoomLevelArgument {
	pub Level:f64,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ProcessMemoryInformationDto {
	pub PrivateBytes:usize,
	pub SharedBytes:usize,
	pub ResidentSetSize:usize,
}

/// A Tauri command to set the zoom level of the main application window.
#[command(rename_all = "PascalCase")]
pub async fn MountainSetZoomLevel(window:Window<Wry>, argument:SetZoomLevelArgument) -> Result<(), String> {
	debug!("[SkyCommandLogic] Setting zoom level to: {}", argument.Level);
	// Use an exponential scale for zoom, similar to VS Code.
	let zoom_factor = 1.2_f64.powf(argument.Level);
	window
		.set_zoom(zoom_factor)
		.map_err(|e| error_utils::RpcErrorString(format!("Failed to set zoom: {}", e), None))
}

/// A Tauri command to fetch the shell Environment variables of the Mountain
/// process.
#[command(rename_all = "PascalCase")]
pub async fn MountainFetchShellEnv() -> Result<HashMap<String, String>, String> {
	debug!("[SkyCommandLogic] Fetching shell Environment.");
	Ok(std::env::vars().collect())
}

/// A Tauri command to get memory usage information for the main Mountain
/// process.
#[command(rename_all = "PascalCase")]
pub async fn MountainGetProcessMemoryInfo() -> Result<ProcessMemoryInformationDto, String> {
	debug!("[SkyCommandLogic] Getting process memory info.");
	let mut system_info = System::new();
	system_info.refresh_processes_specifics(ProcessRefreshKind::new().with_memory());
	let current_pid = sysinfo::get_current_pid()
		.map_err(|e| error_utils::RpcErrorString(format!("Failed to get PID: {}", e), None))?;

	if let Some(process_info) = system_info.process(current_pid) {
		Ok(ProcessMemoryInformationDto {
			// Calculations to convert KB from sysinfo to Bytes.
			PrivateBytes:(process_info.virtual_memory().saturating_sub(process_info.memory()) * 1024) as usize,
			SharedBytes:0, // sysinfo doesn't easily provide this on all platforms.
			ResidentSetSize:(process_info.memory() * 1024) as usize,
		})
	} else {
		Err(error_utils::RpcErrorString(
			"Could not find current process info.".to_string(),
			None,
		))
	}
}
