// ---------------------------------------------------------------------------------------------
// Mountain Sky-Specific Command Handlers (handlers/sky_commands.rs)
// --------------------------------------------------------------------------------------------
// This module implements Tauri commands specifically invoked by the Sky
// frontend for actions that are closely tied to the native application
// environment or window management, which Mountain provides. These are
// typically not general "effects" but direct interactions.
//
// Examples:
// - Setting window zoom level.
// - Fetching shell environment variables of the Mountain process.
// - Getting process memory information for the Mountain process.
// --------------------------------------------------------------------------------------------

use std::collections::HashMap; // For shell_env

use log::{debug, error, warn};
use serde::{Deserialize, Serialize}; // Added Serialize for ProcessMemoryInfoDto
// For mountain_get_process_memory_info
use sysinfo::{Pid, ProcessExt, ProcessRefreshKind, System, SystemExt};
use tauri::{AppHandle, Manager, Runtime, Window, Wry}; // Wry for default runtime

/// Argument for the `mountain_set_zoom_level` command.
#[derive(Deserialize, Debug)]
struct SetZoomLevelArgument {
	level:f64, // Exponent for zoom calculation (e.g., 0=default, 1=20% larger)
}

/// Tauri command to set the zoom level of the invoking window.
/// Sky sends an exponent value (like VS Code's internal level), which is then
/// converted to a zoom factor (e.g., 1.2^level).
#[tauri::command]
pub async fn mountain_set_zoom_level(
	window:Window<Wry>, // The window that invoked this command
	args:SetZoomLevelArgument,
) -> Result<(), String> {
	debug!(
		"[SkyCommands] mountain_set_zoom_level: Window='{}', Level={}",
		window.label(),
		args.level
	);

	// VS Code's zoom levels are often powers of 1.2.
	// Level 0 -> Factor 1.0 (1.2^0)
	// Level 1 -> Factor 1.2 (1.2^1)
	// Level -1 -> Factor 1/1.2 (1.2^-1)
	let zoom_factor = 1.2_f64.powf(args.level);

	match window.set_zoom(zoom_factor) {
		Ok(_) => {
			debug!(
				"[SkyCommands] Zoom level set to {} (factor {}) for window '{}'",
				args.level,
				zoom_factor,
				window.label()
			);
			Ok(())
		},
		Err(e) => {
			let err_msg = format!("Failed to set zoom level for window '{}': {}", window.label(), e);
			error!("[SkyCommands] {}", err_msg);
			Err(err_msg)
		},
	}
}

/// Tauri command to fetch the shell environment variables of the Mountain
/// (Tauri main) process. For security, it's advisable to filter these variables
/// before sending them to the frontend, but this MVP version returns all of
/// them.
#[tauri::command]
pub async fn mountain_fetch_shell_env(
	_app_handle:AppHandle<Wry>, // Not strictly needed if just reading std::env::vars
) -> Result<HashMap<String, String>, String> {
	debug!("[SkyCommands] mountain_fetch_shell_env called.");

	let shell_env:HashMap<String, String> = std::env::vars().collect();

	// Security Note: Sending all environment variables to the frontend can be a
	// risk. Consider filtering to an allow-list of known safe/necessary variables.
	// Example:
	// let allow_list = ["PATH", "HOME", "LANG", "SHELL", "VSCODE_GIT_IPC_HANDLE"];
	// let filtered_env: HashMap<String, String> = shell_env
	//     .into_iter()
	//     .filter(|(key, _)| allow_list.contains(&key.as_str()))
	//     .collect();
	// debug!("[SkyCommands] Returning shell env with {} variables (filtered).",
	// filtered_env.len()); Ok(filtered_env)

	debug!(
		"[SkyCommands] Returning current process environment ({} variables). Consider filtering.",
		shell_env.len()
	);
	Ok(shell_env)
}

/// DTO for process memory information, mimicking Electron's
/// `ProcessMemoryInfo`.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMemoryInfoDto {
	/// Amount of memory not shared by other processes (approximate).
	pub private_bytes:usize,
	/// Amount of memory shared between processes (hard to determine accurately,
	/// often stubbed).
	pub shared_bytes:usize,
	/// Resident Set Size: Actual physical memory used by the process.
	pub resident_set_size:usize,
}

/// Tauri command to get memory usage information for the Mountain (Tauri main)
/// process. Uses the `sysinfo` crate to gather this data. This reflects the
/// main backend process, not necessarily the detailed memory usage of the
/// webview/renderer content.
#[tauri::command]
pub async fn mountain_get_process_memory_info(_app_handle:AppHandle<Wry>) -> Result<ProcessMemoryInfoDto, String> {
	debug!("[SkyCommands] mountain_get_process_memory_info called.");

	let mut sys = System::new_all();
	// Refresh only process information needed for memory.
	sys.refresh_processes_specifics(ProcessRefreshKind::new().with_memory());

	let current_pid = match sysinfo::get_current_pid() {
		Ok(pid) => pid,
		Err(e) => {
			let err_msg = format!("Failed to get current PID: {}", e);
			error!("[SkyCommands] {}", err_msg);
			return Err(err_msg);
		},
	};

	if let Some(process) = sys.process(current_pid) {
		let memory_kb = process.memory(); // Resident Set Size in KB
		let virtual_memory_kb = process.virtual_memory(); // Virtual memory size in KB

		debug!(
			"[SkyCommands] Main process PID {}: Memory RSS={}KB, Virtual={}KB",
			current_pid, memory_kb, virtual_memory_kb
		);

		// Approximations:
		// private_bytes: Often estimated. `virtual_memory - resident_set_size` can be a
		// rough heuristic                but isn't strictly "private" memory. For
		// simplicity, some use virtual_memory directly                or a portion of
		// it if resident set is already accounted for. shared_bytes: Very difficult
		// to determine accurately cross-platform without deep OS specifics.
		//               Often stubbed or omitted.
		let resident_set_bytes = (memory_kb * 1024) as usize;
		let private_bytes_approx = (virtual_memory_kb.saturating_sub(memory_kb) * 1024) as usize;

		Ok(ProcessMemoryInfoDto {
			private_bytes:private_bytes_approx,
			shared_bytes:0, // Stubbed as it's hard to get reliably
			resident_set_size:resident_set_bytes,
		})
	} else {
		warn!(
			"[SkyCommands] Could not find current process (PID: {}) info in sysinfo. Returning stubbed memory info.",
			current_pid
		);
		Ok(ProcessMemoryInfoDto { private_bytes:0, shared_bytes:0, resident_set_size:0 })
	}
}
