//! # ConfigurationDataCommand
//!
//! Handles configuration data retrieval and saving for Wind frontend.
//!
//! ## RESPONSIBILITIES
//!
//! ### Configuration Data Access
//! - Retrieve configuration data for Wind
//! - Save configuration data from Wind
//! - Validate configuration structure
//! - Handle partial updates
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Configuration data CRUD endpoint
//!
//! ### Dependencies
//! - crate::IPC::ConfigurationBridge: Data persistence
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Gets and sets configuration data
//! - Tauri IPC handler: Routes configuration operations
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Validate configuration structure on save
//! - Sanitize user input to prevent injection
//! - Prevent modification of protected keys
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Configuration reads are cached when possible
//! - Writes trigger persistence, may be slower
//! - Consider debouncing save operations

use serde_json::Value;
use tauri::AppHandle;

/// Get configuration data for Wind frontend.
///
/// Retrieves the current configuration data for display/editing in Wind.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns configuration data as JSON, or an error string.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be loaded
/// - File system errors occur
#[tauri::command]
pub async fn GetConfigurationData(app: AppHandle) -> Result<Value, String> {
	crate::IPC::ConfigurationBridge::get_configuration_data(app).await
}

/// Save configuration data from Wind frontend.
///
/// Persists configuration data provided by the Wind frontend.
///
/// # Arguments
///
/// * `app` - Tauri application handle
/// * `config_data` - JSON object containing configuration data to save
///
/// # Returns
///
/// Returns success or an error string.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration validation fails
/// - File system errors occur when persisting
#[tauri::command]
pub async fn SaveConfigurationData(app: AppHandle, config_data: Value) -> Result<(), String> {
	crate::IPC::ConfigurationBridge::save_configuration_data(app, config_data).await
}
