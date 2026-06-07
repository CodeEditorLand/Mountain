//! # ConfigurationSyncCommand
//!
//! Synchronizes configuration across the application.
//!
//! ## RESPONSIBILITIES
//!
//! ### Configuration Synchronization
//! - Trigger configuration synchronization
//! - Coordinate between configuration sources
//! - Return sync status and conflicts
//! - Handle offline/online state
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Configuration synchronization endpoint
//!
//! ### Dependencies
//! - crate::IPC::ConfigurationBridge: Synchronization logic
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Triggers configuration sync
//! - Tauri IPC handler: Routes sync requests
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Validate configuration during sync
//! - Prevent sync of sensitive data without encryption
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Sync may involve network operations
//! - Implement conflict detection early
//! - Consider incremental sync for large configs

use serde_json::Value;
use tauri::AppHandle;

/// Synchronize configuration.
///
/// This command triggers a configuration synchronization between different
/// configuration sources through the ConfigurationBridge.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns a JSON response indicating sync status, or an error string.
///
/// # Errors
///
/// Returns an error if:
/// - Sync process fails
/// - Network connectivity issues
/// - Configuration conflicts cannot be resolved
#[tauri::command]
pub async fn MountainSynchronizeConfiguration(app_handle:AppHandle) -> Result<Value, String> {
	crate::IPC::ConfigurationBridge::mountain_synchronize_configuration(app_handle).await
}
