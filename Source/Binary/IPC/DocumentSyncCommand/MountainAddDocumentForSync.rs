//! `DocumentSyncCommand::MountainAddDocumentForSync`

use serde_json::Value;
use tauri::AppHandle;
use crate::dev_log;

/// Add document for sync.
///
/// Registers a document for synchronization with remote collaborators.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
/// * `document_data` - JSON object with document_id and file_path fields
///
/// # Returns
///
/// Returns success JSON or an error string.
///
/// # Errors
///
/// Returns an error if:
/// - Required fields missing
/// - Document registration fails
#[tauri::command]
pub async fn Fn(app_handle:AppHandle, document_data:Value) -> Result<Value, String> {
	let DocumentId = document_data["document_id"]
		.as_str()
		.ok_or_else(|| {
			dev_log!("ipc", "error: [IPC] [Sync] Missing document_id in document_data");
			"Missing document_id"
		})?
		.to_string();

	let FilePath = document_data["file_path"]
		.as_str()
		.ok_or_else(|| {
			dev_log!("ipc", "error: [IPC] [Sync] Missing file_path in document_data");
			"Missing file_path"
		})?
		.to_string();

	crate::IPC::WindAdvancedSync::MountainAddDocumentForSync(app_handle, DocumentId, FilePath)
		.await
		.map_err(|Error| {
			dev_log!("ipc", "error: [IPC] [Sync] Failed to add document for sync: {}", Error);
			Error.to_string()
		})
		.map(|_| Value::Null)
}
