
//! Tauri command - debug-adapter-protocol service health probe. Stub
//! returns `false`; pending DAP wire-up.

#[tauri::command]
pub async fn cocoon_debug_service_health() -> Result<bool, String> { Ok(false) }
