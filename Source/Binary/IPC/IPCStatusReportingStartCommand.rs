//! # IPCStatusReportingStartCommand
//!
//! Starts periodic IPC status reporting.
//!
//! ## RESPONSIBILITIES
//!
//! ### Reporting Control
//! - Start periodic status reporting
//! - Configure reporting interval
//! - Enable status monitoring
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Status reporting control endpoint
//!
//! ### Dependencies
//! - crate::IPC::StatusReporter: Reporting logic
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Starts status monitoring
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Rate limit reporting intervals
//! - Validate authorization before starting
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Periodic reporting has ongoing overhead
//! - Default interval should be reasonable (60s)

use serde_json::Value;
use tauri::AppHandle;

/// Start IPC status reporting.
///
/// Enables periodic IPC status reporting for monitoring.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns confirmation JSON, or an error string.
///
/// # Errors
///
/// Returns an error if reporting cannot be started.
#[tauri::command]
pub async fn MountainStartIPCStatusReporting(app_handle: AppHandle) -> Result<Value, String> {
	crate::IPC::StatusReporter::mountain_start_ipc_status_reporting(app_handle, 60).await
}
