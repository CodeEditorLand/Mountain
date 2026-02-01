//! # ConfigurationUpdateCommand
//!
//! Updates configuration from Wind frontend.
//!
//! ## RESPONSIBILITIES
//!
//! ### Configuration Update
//! - Accept configuration updates from Wind
//! - Delegate to ConfigurationBridge for processing
//! - Validate configuration structure
//! - Return update confirmation
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Bridge for Wind configuration updates
//!
//! ### Dependencies
//! - crate::IPC::ConfigurationBridge: Configuration management
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Sends configuration updates
//! - Tauri IPC handler: Routes update requests
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Validate configuration structure before applying
//! - Prevent modification of protected configuration keys
//! - Sanitize user-provided values
//!
//! ## PERFORMANCE
//!
//!### Considerations
//! - Configuration updates trigger I/O operations
//! - Consider debouncing rapid updates

use serde_json::Value;
use tauri::AppHandle;

/// Update configuration from Wind.
///
/// This command accepts configuration updates from the Wind frontend
/// and processes them through the ConfigurationBridge.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
/// * `config` - JSON object containing configuration updates
///
/// # Returns
///
/// Returns a JSON response confirming the update, or an error string.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration structure is invalid
/// - Update cannot be persisted
#[tauri::command]
pub async fn MountainUpdateConfigurationFromWind(
	app_handle: AppHandle,
	config: Value,
) -> Result<Value, String> {
	crate::IPC::ConfigurationBridge::mountain_update_configuration_from_wind(app_handle, config).await
}
