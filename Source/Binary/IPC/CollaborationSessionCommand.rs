//! # CollaborationSessionCommand
//!
//! Manages collaboration sessions for multi-user editing.
//!
//! ## RESPONSIBILITIES
//!
//! ### Session Management
//! - Create new collaboration sessions
//! - Get existing collaboration sessions
//! - Validate session data
//! - Handle session credentials
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Collaboration feature endpoint
//!
//! ### Dependencies
//! - crate::IPC::AdvancedFeatures: Session management
//! - tauri: IPC framework
//! - serde_json: JSON serialization
//!
//! ### Dependents
//! - Wind frontend: Creates/queries sessions
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Validate session data structure
//! - Sanitize session identifiers
//! - Implement access control
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - Session operations should be fast
//! - Consider connection pooling for active sessions

use log::error;
use serde_json::{Value, to_value};
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
/// Returns session info JSON, or an error string.
///
/// # Errors
///
/// Returns an error if:
/// - Session data is invalid
/// - Session creation fails
#[tauri::command]
pub async fn MountainCreateCollaborationSession(app_handle:AppHandle, session_data:Value) -> Result<Value, String> {
	let session = crate::IPC::AdvancedFeatures::mountain_create_collaboration_session(app_handle, session_data).await?;
	to_value(&session).map_err(|e| format!("Failed to serialize collaboration session: {}", e))
}

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
pub async fn MountainGetCollaborationSessions(app_handle:AppHandle) -> Result<Value, String> {
	let sessions = crate::IPC::AdvancedFeatures::mountain_get_collaboration_sessions(app_handle).await;
	to_value(&sessions).map_err(|e| format!("Failed to serialize collaboration sessions: {}", e))
}
