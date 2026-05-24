//! # StatusGetCommand
//!
//! Retrieves the current Mountain IPC status.
//!
//! ## RESPONSIBILITIES
//!
//! ### Status Query
//! - Get current IPC server status
//! - Query active connections and message metrics
//! - Return structured status information
//! - Handle error conditions gracefully
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC command in Binary subsystem
//! - Diagnostic endpoint for IPC health
//!
//! ### Dependencies
//! - crate::IPC::TauriIPCServer_Old: Status retrieval
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Queries status for diagnostics
//! - Development tools: Monitor IPC health
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Status information is read-only, no modification
//! - Avoid exposing sensitive connection details
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Status query is typically fast, in-memory operation
//! - Consider rate limiting if called frequently

use serde_json::Value;
use tauri::AppHandle;

use crate::dev_log;

/// Get Mountain IPC status.
///
/// This command retrieves the current status of the IPC server including
/// connection information, message statistics, and operational state.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns a JSON object containing the IPC status on success,
/// or an error string on failure.
///
/// # Errors
///
/// Returns an error if:
/// - Status cannot be retrieved from IPC server
#[tauri::command]
pub async fn Fn(app_handle:AppHandle) -> Result<Value, String> {
	let Status = crate::IPC::TauriIPCServer_Old::MountainIpcGetStatus(app_handle)
		.await
		.map_err(|Error| {
			dev_log!("ipc", "error: [IPC] [Command] Failed to get IPC status: {}", Error);
			Error.to_string()
		})?;

	Ok(serde_json::to_value(Status).map_err(|E| e.to_string())?)
}
