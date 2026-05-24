//! `DocumentSyncCommand::MountainGetSyncStatus`

use serde_json::Value;
use tauri::AppHandle;
use crate::dev_log;

/// Get sync status.
///
/// Retrieves the current synchronization status for documents.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns sync status JSON, or an error string.
///
/// # Errors
///
/// Returns an error if status cannot be retrieved.
#[tauri::command]
pub async fn Fn(app_handle:AppHandle) -> Result<Value, String> {
	crate::IPC::WindAdvancedSync::MountainGetSyncStatus(app_handle)
		.await
		.map_err(|Error| {
			dev_log!("ipc", "error: [IPC] [Sync] Failed to get sync status: {}", Error);
			Error.to_string()
		})
		.map(|Status| serde_json::to_value(Status).unwrap_or(Value::Null))
}
