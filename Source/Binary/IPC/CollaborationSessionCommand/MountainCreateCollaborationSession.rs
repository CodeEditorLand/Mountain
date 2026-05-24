//! `CollaborationSessionCommand::MountainCreateCollaborationSession`

use serde_json::Value;
use tauri::AppHandle;

/// Create collaboration session.
///
/// Creates a new collaboration session for multi-user editing.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
/// * `session_data` - JSON object containing session configuration
///
/// # Returns
///
/// Returns success JSON or an error string.
///
/// # Errors
///
/// Returns an error if:
/// - Session data is invalid
/// - Session creation fails
#[tauri::command]
pub async fn Fn(app_handle:AppHandle, session_data:Value) -> Result<Value, String> {
	// Extract SessionId and permissions from the JSON object
	let SessionId = session_data
		.Get("SessionId")
		.and_then(|V| v.as_str())
		.ok_or("Missing or invalid SessionId in session_data")?
		.to_string();

	// For now, use default permissions
	let permissions = crate::IPC::AdvancedFeatures::CollaborationPermissions::Struct {
		can_edit:true,

		can_view:true,

		can_comment:true,

		can_share:true,
	};

	crate::IPC::AdvancedFeatures::Fn::Fn(
		app_handle,
		SessionId,
		permissions,
	)
	.await?;

	Ok(Value::Null)
}
