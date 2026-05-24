//! `ConfigurationDataCommand::SaveConfigurationData`

use serde_json::Value;
use tauri::AppHandle;

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
pub async fn Fn(app:AppHandle, config_data:Value) -> Result<(), String> {
	crate::IPC::ConfigurationBridge::SaveConfigurationData(app, config_data).await
}
