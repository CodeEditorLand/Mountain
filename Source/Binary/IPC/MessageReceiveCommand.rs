//! # MessageReceiveCommand
//!
//! Handles receiving messages from Wind through IPC.
//!
//! ## RESPONSIBILITIES
//!
//! ### Message Reception
//! - Receive JSON messages from Wind frontend
//! - Parse and validate message structure
//! - Delegate to TauriIPCServer for message processing
//! - Return success/error response to caller
//!
//! ## ARCHITECTURAL ROLE
//!
//! ### Position in Mountain
//! - IPC wrapper command in Binary subsystem
//! - Bridge between frontend and IPC server
//!
//! ### Dependencies
//! - crate::IPC::TauriIPCServer: Message processing
//! - tauri: IPC framework
//! - serde_json: JSON parsing
//!
//! ### Dependents
//! - Wind frontend: Sends messages via this command
//! - Tauri IPC handler: Routes messages to this command
//!
//! ## SECURITY
//!
//! ### Considerations
//! - Validate JSON structure before processing
//! - Malformed JSON should not crash the application
//! - Messages may contain user input; proper parsing required
//!
//! ## PERFORMANCE
//!
//! ### Considerations
//! - JSON parsing has some overhead but is fast for typical messages
//! - Async execution doesn't block main thread
//! - Consider message batching for high-frequency updates

use serde_json::Value;
use tauri::AppHandle;

/// Receive messages from Wind through IPC.
///
/// This command accepts JSON messages from the Wind frontend and delegates
/// them to the TauriIPCServer for processing. The message is first parsed
/// to ensure it has the correct structure.
///
/// # Arguments
///
/// * `app_handle` - Tauri application handle
/// * `message` - JSON value containing the message from Wind
///
/// # Returns
///
/// Returns a JSON response on success, or an error string on failure.
///
/// # Errors
///
/// Returns an error if:
/// - JSON message cannot be parsed into the expected structure
/// - TauriIPCServer processing fails
#[tauri::command]
pub async fn MountainIPCReceiveMessage(app_handle:AppHandle, message:Value) -> Result<Value, String> {
	crate::IPC::TauriIPCServer::mountain_ipc_receive_message(
		app_handle,
		serde_json::from_value(message).map_err(|e| e.to_string())?,
	)
	.await
}
