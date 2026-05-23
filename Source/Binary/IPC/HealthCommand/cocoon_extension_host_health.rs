
//! Tauri command - extension-host (Cocoon) health probe. Wire name
//! kept snake_case to match Wind's `SharedProcessProxy` invoker.
//!
//! ## Stub
//!
//! Wire to real Cocoon gRPC health check when Cocoon is spawned.
//! Currently returns `false` unconditionally.

#[tauri::command]
pub async fn cocoon_extension_host_health() -> Result<bool, String> { Ok(false) }
