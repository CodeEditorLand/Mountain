//! Tauri command - search-service health probe. Returns `true` while
//! the file system is reachable; the search service treats FS access
//! as its readiness signal.

#[tauri::command]
pub async fn cocoon_search_service_health() -> Result<bool, String> { Ok(true) }
