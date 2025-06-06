
// Defines command handlers for requests originating from the Sky frontend that
// are specific to the workbench or process, such as setting zoom or fetching
// memory info.

#![allow(non_snake_case, non_camel_case_types)]

use std::collections::HashMap;

use log::{debug, error, warn};
use serde::{Deserialize, Serialize};
use sysinfo::{Pid, ProcessExt, ProcessRefreshKind, System, SystemExt};
use tauri::{AppHandle, Manager, Runtime, Window, Wry};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SetZoomLevelArgument {
	Level:f64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ProcessMemoryInformationDto {
	pub PrivateBytes:usize,
	pub SharedBytes:usize,
	pub ResidentSetSize:usize,
}

/// Sets the zoom level of the main application window.
#[tauri::command]
pub async fn MountainSetZoomLevel(Window:Window<Wry>, Argument:SetZoomLevelArgument) -> Result<(), String> {
	debug!(
		"[SkyCommands] MountainSetZoomLevel: Window='{}', Level={}",
		Window.label(),
		Argument.Level
	);

	// Zoom factor calculation matches VS Code's behavior (1.2^level).
	let ZoomFactor = 1.2_f64.powf(Argument.Level);

	Window.set_zoom(ZoomFactor).map_err(|TauriError| {
		let ErrorMessage = format!("Failed to set zoom level for window '{}': {}", Window.label(), TauriError);
		error!("[SkyCommands] {}", ErrorMessage);
		ErrorMessage
	})
}

/// Fetches the shell environment variables of the current process.
#[tauri::command]
pub async fn MountainFetchShellEnv(_ApplicationHandle:AppHandle<Wry>) -> Result<HashMap<String, String>, String> {
	debug!("[SkyCommands] MountainFetchShellEnv called.");

	let ShellEnvironment:HashMap<String, String> = std::env::vars().collect();

	debug!(
		"[SkyCommands] Returning current process environment ({} variables). Consider filtering for security.",
		ShellEnvironment.len()
	);
	Ok(ShellEnvironment)
}

/// Retrieves memory usage information for the current Mountain process.
#[tauri::command]
pub async fn MountainGetProcessMemoryInfo(
	_ApplicationHandle:AppHandle<Wry>,
) -> Result<ProcessMemoryInformationDto, String> {
	debug!("[SkyCommands] MountainGetProcessMemoryInfo called.");

	let mut SystemInfo = System::new_all();
	SystemInfo.refresh_processes_specifics(ProcessRefreshKind::new().with_memory());

	let CurrentPid = match sysinfo::get_current_pid() {
		Ok(Pid) => Pid,
		Err(Error) => {
			let ErrorMessage = format!("Failed to get current PID: {}", Error);
			error!("[SkyCommands] {}", ErrorMessage);
			return Err(ErrorMessage);
		},
	};

	if let Some(ProcessInfo) = SystemInfo.process(CurrentPid) {
		let MemoryKilobytes = ProcessInfo.memory();
		let VirtualMemoryKilobytes = ProcessInfo.virtual_memory();

		debug!(
			"[SkyCommands] Main process PID {}: Memory RSS={}KB, Virtual={}KB",
			CurrentPid, MemoryKilobytes, VirtualMemoryKilobytes
		);

		let ResidentSetBytes = (MemoryKilobytes * 1024) as usize;
		// This is a rough approximation of private bytes.
		let PrivateBytesApprox = (VirtualMemoryKilobytes.saturating_sub(MemoryKilobytes) * 1024) as usize;

		Ok(ProcessMemoryInformationDto {
			PrivateBytes:PrivateBytesApprox,
			SharedBytes:0, // sysinfo doesn't directly provide shared memory on all platforms.
			ResidentSetSize:ResidentSetBytes,
		})
	} else {
		warn!(
			"[SkyCommands] Could not find current process info (PID: {}) in sysinfo. Returning stubbed memory info.",
			CurrentPid
		);
		Ok(ProcessMemoryInformationDto { PrivateBytes:0, SharedBytes:0, ResidentSetSize:0 })
	}
}
