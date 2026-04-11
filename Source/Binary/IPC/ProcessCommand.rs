//! # ProcessCommand
//!
//! Tauri commands for Wind's ProcessPolyfill.
//! These are invoked directly (not through MountainIPCInvoke) as separate
//! Tauri commands: process:get_exec_path, process:get_platform, etc.

use std::collections::HashMap;

use serde_json::{Value, json};

/// Get the executable path of the current process.
#[tauri::command]
pub async fn process_get_exec_path() -> Result<String, String> {
	std::env::current_exe()
		.map(|P| P.to_string_lossy().to_string())
		.map_err(|E| format!("Failed to get exec path: {}", E))
}

/// Get the current platform identifier.
#[tauri::command]
pub async fn process_get_platform() -> Result<String, String> {
	Ok(match std::env::consts::OS {
		"macos" => "darwin",
		"windows" => "win32",
		"linux" => "linux",
		Other => Other,
	}
	.to_string())
}

/// Get the CPU architecture.
#[tauri::command]
pub async fn process_get_arch() -> Result<String, String> { Ok(std::env::consts::ARCH.to_string()) }

/// Get the process ID.
#[tauri::command]
pub async fn process_get_pid() -> Result<u32, String> { Ok(std::process::id()) }

/// Get the shell environment variables.
/// Returns the full environment of the Mountain process, which inherits
/// the user's shell environment on all platforms.
#[tauri::command]
pub async fn process_get_shell_env() -> Result<HashMap<String, String>, String> { Ok(std::env::vars().collect()) }

/// Get process memory information.
/// Returns private, shared, and residentSet memory in bytes.
#[tauri::command]
pub async fn process_get_memory_info() -> Result<Value, String> {
	#[cfg(target_os = "macos")]
	{
		// macOS: use mach_task_basic_info or fallback to ps
		let Output = std::process::Command::new("ps")
			.args(["-o", "rss=,vsz=", "-p", &std::process::id().to_string()])
			.output();

		match Output {
			Ok(Out) => {
				let Text = String::from_utf8_lossy(&Out.stdout);
				let Parts:Vec<&str> = Text.split_whitespace().collect();
				let Rss = Parts.first().and_then(|V| V.parse::<u64>().ok()).unwrap_or(0) * 1024;
				let Vsz = Parts.get(1).and_then(|V| V.parse::<u64>().ok()).unwrap_or(0) * 1024;
				Ok(json!({
					"private": Rss,
					"shared": 0,
					"residentSet": Rss
				}))
			},
			Err(_) => Ok(json!({ "private": 0, "shared": 0, "residentSet": 0 })),
		}
	}

	#[cfg(target_os = "windows")]
	{
		// Windows: read from /proc/self/status equivalent or tasklist
		let Output = std::process::Command::new("tasklist")
			.args(["/FI", &format!("PID eq {}", std::process::id()), "/FO", "CSV", "/NH"])
			.output();

		match Output {
			Ok(Out) => {
				let Text = String::from_utf8_lossy(&Out.stdout);
				// Parse "Image Name","PID","Session Name","Session#","Mem Usage"
				let MemStr = Text.split(',').nth(4).unwrap_or("\"0 K\"");
				let MemKb:u64 = MemStr
					.chars()
					.filter(|C| C.is_ascii_digit())
					.collect::<String>()
					.parse()
					.unwrap_or(0);
				let MemBytes = MemKb * 1024;
				Ok(json!({
					"private": MemBytes,
					"shared": 0,
					"residentSet": MemBytes
				}))
			},
			Err(_) => Ok(json!({ "private": 0, "shared": 0, "residentSet": 0 })),
		}
	}

	#[cfg(target_os = "linux")]
	{
		// Linux: read /proc/self/statm
		match tokio::fs::read_to_string("/proc/self/statm").await {
			Ok(Content) => {
				let Parts:Vec<&str> = Content.split_whitespace().collect();
				let PageSize:u64 = 4096; // typical page size
				let Vsz = Parts.first().and_then(|V| V.parse::<u64>().ok()).unwrap_or(0) * PageSize;
				let Rss = Parts.get(1).and_then(|V| V.parse::<u64>().ok()).unwrap_or(0) * PageSize;
				let Shared = Parts.get(2).and_then(|V| V.parse::<u64>().ok()).unwrap_or(0) * PageSize;
				Ok(json!({
					"private": Rss.saturating_sub(Shared),
					"shared": Shared,
					"residentSet": Rss
				}))
			},
			Err(_) => Ok(json!({ "private": 0, "shared": 0, "residentSet": 0 })),
		}
	}

	#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
	{
		Ok(json!({ "private": 0, "shared": 0, "residentSet": 0 }))
	}
}
