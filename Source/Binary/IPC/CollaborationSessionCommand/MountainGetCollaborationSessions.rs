//! `CollaborationSessionCommand::MountainGetCollaborationSessions`

use serde_json::Value;
use tauri::AppHandle;

/// Get collaboration sessions.
///
/// Retrieves existing collaboration sessions.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
///
/// # Returns
///
/// Returns sessions JSON, or an error string.
///
/// # Errors
///
/// Returns an error if sessions cannot be retrieved.
#[tauri::command]
pub async fn Fn(app_handle:AppHandle) -> Result<Value, String> {
	let sessions =
		crate::IPC::AdvancedFeatures::Fn::Fn(app_handle)
			.await;

	serde_json::to_value(&sessions).map_err(|E| format!("Failed to serialize collaboration sessions: {}", e))
}
