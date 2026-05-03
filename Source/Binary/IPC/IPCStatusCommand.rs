//! # IPCStatusCommand
//!
//! Retrieves IPC status information for monitoring.
//!
//! ## RESPONSIBILITIES
//!
//! ### Status Reporting
//! - Get current IPC server status
//! - Report connection metrics
//! - Return operational state information
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Status reporter endpoint
//!
//! ### Dependencies
//! - crate::IPC::StatusReporter: Status tracking
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Queries IPC status
//! - DevTools: Monitoring IPC health
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Status is read-only, no security impact
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Status queries are fast, in-memory operations

use serde_json::Value;
use tauri::AppHandle;

/// Get IPC status.
///
/// Retrieves the current status of the IPC system including
/// connection state and operational metrics.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns IPC status JSON, or an error string.
///
/// # Errors
///
/// Returns an error if status cannot be retrieved.
#[tauri::command]
pub async fn MountainGetIPCStatus(app_handle:AppHandle) -> Result<Value, String> {
	crate::IPC::StatusReporter::mountain_get_ipc_status::mountain_get_ipc_status(app_handle).await
}
