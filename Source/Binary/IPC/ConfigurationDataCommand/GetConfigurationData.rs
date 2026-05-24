//! `ConfigurationDataCommand::GetConfigurationData`

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
pub async fn Fn(app:AppHandle) -> Result<Value, String> {
	crate::IPC::ConfigurationBridge::GetConfigurationData(app).await
}
