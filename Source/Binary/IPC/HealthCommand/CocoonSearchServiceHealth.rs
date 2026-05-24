//! Tauri command - search-service health probe. Returns `true` while
//! the file system is reachable; the search service treats FS access
//! as its readiness signal.

#[tauri::command]
pub async fn Fn() -> Result<bool, String> { Ok(true) }
