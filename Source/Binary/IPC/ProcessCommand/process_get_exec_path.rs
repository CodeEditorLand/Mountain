//! Tauri command - return the absolute path of the running Mountain
//! executable. Wire identifier kept snake_case to match Wind's
//! `ProcessPolyfill` invoker.

#[tauri::command]
pub async fn process_get_exec_path() -> Result<String, String> {

	std::env::current_exe()
		.map(|P| P.to_string_lossy().to_string())
		.map_err(|E| format!("Failed to get exec path: {}", E))
}
