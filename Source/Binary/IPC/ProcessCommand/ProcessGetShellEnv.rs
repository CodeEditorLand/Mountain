//! Tauri command - return the full environment of the Mountain
//! process. The process inherits the user's shell environment on every
//! supported platform, so this is `process.env` for renderer code.

use std::collections::HashMap;

#[tauri::command]
pub async fn Fn() -> Result<HashMap<String, String>, String> { Ok(std::env::vars().collect()) }
