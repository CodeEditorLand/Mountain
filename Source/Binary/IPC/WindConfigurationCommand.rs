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

use serde_json::{Value, to_value};
use tauri::AppHandle;

/// Get Wind desktop configuration.
///
/// Retrieves the Wind desktop configuration for frontend initialization.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns Wind configuration JSON, or an error string.
///
/// # Errors
///
/// Returns an error if:
/// - Configuration cannot be loaded
/// - Serialization fails
#[tauri::command]
pub async fn Fn(app_handle:AppHandle) -> Result<Value, String> {
	let config = crate::IPC::ConfigurationBridge::Fn(app_handle).await?;

	to_value(&config).map_err(|E| format!("Failed to serialize configuration: {}", e))
}
