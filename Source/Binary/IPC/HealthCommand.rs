//! # HealthCommand
//!
//! Tauri commands for Wind's SharedProcessProxy health checks.
//! These are invoked directly as Tauri commands from the frontend.

/// Check extension host (Cocoon) health.
/// Returns true if Cocoon is connected via gRPC, false otherwise.
#[tauri::command]
pub async fn cocoon_extension_host_health() -> Result<bool, String> {
	// TODO: Wire to real Cocoon gRPC health check when Cocoon is spawned
	Ok(false)
}

/// Check search service health.
#[tauri::command]
pub async fn cocoon_search_service_health() -> Result<bool, String> {
	// Search is available when file system is accessible
	Ok(true)
}

/// Check debug service health.
#[tauri::command]
pub async fn cocoon_debug_service_health() -> Result<bool, String> {
	// Debug adapter protocol - not yet wired
	Ok(false)
}

/// Generic shared process service health check.
#[tauri::command]
pub async fn shared_process_service_health(service:String) -> Result<bool, String> {
	match service.as_str() {
		"storage" => Ok(true), // Storage is always available (file-backed)
		"update" => Ok(true),  // Update service always reports idle
		"search" => Ok(true),  // Search available via file system
		_ => Ok(false),
	}
}
