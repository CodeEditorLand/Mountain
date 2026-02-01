//! # WindConfigurationCommand
//!
//! Retrieves Wind desktop configuration via IPC.
//!
//! ## RESPONSIBILITIES
//!
//! ### Configuration Retrieval
//! - Get Wind desktop configuration from ConfigurationBridge
//! - Return structured configuration data
//! - Handle configuration retrieval errors
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Bridge to ConfigurationBridge for Wind config
//!
//! ### Dependencies
//! - crate::IPC::ConfigurationBridge: Configuration management
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Retrieves desktop configuration
//! - Tauri IPC handler: Routes configuration requests
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Configuration may contain sensitive information
//! - Ensure access control for sensitive configuration keys
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Configuration is typically cached, fast retrieval
//! - Consider caching in memory for frequently accessed config

use serde_json::Value;
use tauri::AppHandle;

/// Get Wind desktop configuration.
///
/// This command retrieves the desktop configuration for Wind from the
/// ConfigurationBridge module.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns a JSON object containing the Wind desktop configuration,
/// or an error string on failure.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be retrieved
/// - Configuration is malformed
#[tauri::command]
pub async fn MountainGetWindDesktopConfiguration(app_handle: AppHandle) -> Result<Value, String> {
	crate::IPC::ConfigurationBridge::mountain_get_wind_desktop_configuration(app_handle).await
}
