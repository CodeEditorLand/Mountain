use std::collections::HashMap;

use log::debug;
use serde::Deserialize;
use sysinfo::{ProcessExt, ProcessRefreshKind, System, SystemExt};
use tauri::{AppHandle, Window, Wry, command};

/// @module SkyCommandsLogic
/// @description Contains the logic for Tauri commands invoked directly by the
/// Sky frontend for window management and system information queries.
use crate::handlers::error_utils;

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
pub async fn MountainSetZoomLevel(Window:Window<Wry>, Argument:SetZoomLevelArgument) -> Result<(), String> {
	debug!("[SkyCommands] Setting zoom level to: {}", Argument.Level);
	// Use an exponential scale for zoom, similar to VS Code.
	let ZoomFactor = 1.2_f64.powf(Argument.Level);
	Window
		.set_zoom(ZoomFactor)
		.map_err(|e| error_utils::RpcErrorString(format!("Failed to set zoom: {}", e), None))
}

/// A Tauri command to fetch the shell environment variables of the Mountain
/// process.
#[command(rename_all = "PascalCase")]
pub async fn MountainFetchShellEnv() -> Result<HashMap<String, String>, String> {
	debug!("[SkyCommands] Fetching shell environment.");
	Ok(std::env::vars().collect())
}

/// A Tauri command to get memory usage information for the main Mountain
/// process.
#[command(rename_all = "PascalCase")]
pub async fn MountainGetProcessMemoryInfo() -> Result<ProcessMemoryInformationDto, String> {
	debug!("[SkyCommands] Getting process memory info.");
	let mut SystemInfo = System::new_all();
	SystemInfo.refresh_processes_specifics(ProcessRefreshKind::new().with_memory());
	let CurrentPid = sysinfo::get_current_pid()
		.map_err(|e| error_utils::RpcErrorString(format!("Failed to get PID: {}", e), None))?;

	if let Some(ProcessInfo) = SystemInfo.process(CurrentPid) {
		Ok(ProcessMemoryInformationDto {
			// Calculations to convert KB from sysinfo to Bytes.
			PrivateBytes:(ProcessInfo.virtual_memory().saturating_sub(ProcessInfo.memory()) * 1024) as usize,
			SharedBytes:0, // sysinfo doesn't easily provide this on all platforms.
			ResidentSetSize:(ProcessInfo.memory() * 1024) as usize,
		})
	} else {
		Err(error_utils::RpcErrorString(
			"Could not find current process info.".to_string(),
			None,
		))
	}
}
