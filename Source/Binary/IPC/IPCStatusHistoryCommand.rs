//! # IPCStatusHistoryCommand
//!
//! Retrieves historical IPC status information.
//!
//! ## RESPONSIBILITIES
//!
//! ### History Query
//! - Get historical IPC status data
//! - Return status changes over time
//! - Provide metrics timeline
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Status history endpoint
//!
//! ### Dependencies
//! - crate::IPC::StatusReporter: Status history
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Queries IPC history
//! - DevTools: Monitoring trends
//!
//! ## SECURITY
//!
//! ### Considerations
//! - History is read-only, no security impact
//! - Limit history size for privacy
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - History queries may be slower with large datasets
//! - Consider pagination for long history

use serde_json::Value;
use tauri::AppHandle;

/// Get IPC status history.
///
/// Retrieves historical IPC status data for analysis and monitoring.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns history JSON, or an error string.
///
/// # Errors
///
/// Returns an error if history cannot be retrieved.
#[tauri::command]
pub async fn MountainGetIPCStatusHistory(app_handle:AppHandle) -> Result<Value, String> {
	crate::IPC::StatusReporter::mountain_get_ipc_status_history::mountain_get_ipc_status_history(app_handle).await
}
