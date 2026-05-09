#![allow(non_snake_case)]

//! Tauri command - return the Wind/Node-style platform identifier
//! (`darwin` / `win32` / `linux` / fallthrough). Mirrors
//! `process.platform` so renderer code that branches on it works
//! unchanged.

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
