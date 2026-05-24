//! # ConfigurationStatusCommand
//!
//! Retrieves the current configuration status.
//!
//! ## RESPONSIBILITIES
//!
//! ### Status Query
//! - Get current configuration state
//! - Report configuration source status
//! - Return validation results
//! - Indicate pending changes
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Configuration status endpoint
//!
//! ### Dependencies
//! - crate::IPC::ConfigurationBridge: Status queries
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Queries configuration status
//! - Tauri IPC handler: Routes status requests
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Status is read-only, no security impact
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Status query is typically fast, in-memory
//! - Cache results if computed value

use serde_json::Value;
use tauri::AppHandle;

/// Get configuration status.
///
/// This command retrieves the current status of the configuration
/// including validation state and pending changes.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns a JSON object with configuration status, or an error string.
///
/// # Errors
///
/// Returns an error if:
/// - Status cannot be retrieved
#[tauri::command]
pub async fn Fn(app_handle:AppHandle) -> Result<Value, String> {
	crate::IPC::ConfigurationBridge::Fn(app_handle).await
}
