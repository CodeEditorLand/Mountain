
//! Tauri command - return the running process ID via
//! `std::process::id()`.

#[tauri::command]
pub async fn process_get_pid() -> Result<u32, String> { Ok(std::process::id()) }
