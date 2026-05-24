//! Tauri command - debug-adapter-protocol service health probe. Stub
//! returns `false`; pending DAP wire-up.

#[tauri::command]
pub async fn Fn() -> Result<bool, String> { Ok(false) }
