//! # DocumentSyncCommand
//!
//! Handles document synchronization for collaboration features.
//!
//! ## RESPONSIBILITIES
//!
//! ### Document Sync
//! - Add documents for synchronization
//! - Get sync status for documents
//! - Validate document identifiers
//! - Track sync progress
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Document sync endpoint
//!
//! ### Dependencies
//! - crate::IPC::WindAdvancedSync: Document synchronization
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//! - log: Logging framework
//!
//! ### Dependents
//! - Wind frontend: Syncs documents
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Validate file paths to prevent directory traversal
//! - Sanitize document IDs
//! - Check file access permissions
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Sync may involve network/disk I/O
//! - Implement progress reporting for large files
//! - Consider delta sync for large documents

use log::error;
use serde_json::Value;
use tauri::AppHandle;

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
pub async fn MountainAddDocumentForSync(app_handle:AppHandle, document_data:Value) -> Result<Value, String> {
	let DocumentId = document_data["document_id"]
		.as_str()
		.ok_or_else(|| {
			error!("[IPC] [Sync] Missing document_id in document_data");
			"Missing document_id"
		})?
		.to_string();
	let FilePath = document_data["file_path"]
		.as_str()
		.ok_or_else(|| {
			error!("[IPC] [Sync] Missing file_path in document_data");
			"Missing file_path"
		})?
		.to_string();

	crate::IPC::WindAdvancedSync::mountain_add_document_for_sync(app_handle, DocumentId, FilePath)
		.await
		.map_err(|Error| {
			error!("[IPC] [Sync] Failed to add document for sync: {}", Error);
			Error.to_string()
		})
		.map(|_| Value::Null)
}

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
pub async fn MountainGetSyncStatus(app_handle:AppHandle) -> Result<Value, String> {
	crate::IPC::WindAdvancedSync::mountain_get_sync_status(app_handle)
		.await
		.map_err(|Error| {
			error!("[IPC] [Sync] Failed to get sync status: {}", Error);
			Error.to_string()
		})
		.map(|Status| serde_json::to_value(Status).unwrap_or(Value::Null))
}
