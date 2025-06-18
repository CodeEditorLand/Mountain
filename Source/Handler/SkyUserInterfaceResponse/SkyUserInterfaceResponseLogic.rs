// @module SkyUserInterfaceResponseLogic
// @description Contains the handler logic for processing asynchronous
// responses sent from the Sky frontend back to the Mountain backend. This is
// the receiving end of the request-response pattern for UI interactions.

use Common::error::CommonError;
use log::{info, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime, command};

use crate::ApplicationState::ApplicationState::ApplicationState;

/// A Tauri command that resolves a pending UI request. The Sky frontend calls
/// this command when a user has completed an action (e.g., selected a file in a
/// dialog, clicked a button on a message).
///
/// @param request_id - The unique ID of the request being resolved.
/// @param data_value - The success data from the UI (e.g., a file path).
/// @param error_details_value - Error information if the UI operation failed.
#[command(rename_all = "PascalCase")]
pub async fn SkyResolvesUiRequest<R:Runtime>(
	app_handle:AppHandle<R>,
	request_id:String,
	data_value:Option<Value>,
	error_details_value:Option<Value>,
) -> Result<(), String> {
	info!("[SkyUiResponse] Resolving UI request ID: {}", request_id);
	let app_state = app_handle.state::<ApplicationState>();

	// Atomically find and remove the pending request's sender from the map.
	let maybe_sender = app_state.PendingUiRequests.lock().unwrap().remove(&request_id);

	if let Some(sender) = maybe_sender {
		// Construct the result to send back to the awaiting task.
		let result_to_send = match (data_value, error_details_value) {
			// If an error payload is present, the operation failed.
			(_, Some(error_value)) => {
				Err(CommonError::UiInteraction { Reason:format!("Error from Sky UI: {:?}", error_value) })
			},
			// If data is present, the operation succeeded.
			(Some(data), None) => Ok(data),
			// If both are absent, the user cancelled the operation (e.g., closed the dialog).
			(None, None) => Ok(Value::Null),
		};

		// Send the result through the oneshot channel.
		if sender.send(result_to_send).is_err() {
			// This happens if the backend task timed out before the UI responded.
			// It's a warning, not a critical error.
			warn!("[SkyUiResponse] UI request {} timed out before Sky responded.", request_id);
		}
	} else {
		// This can happen if the request timed out and was already removed from the
		// map.
		warn!(
			"[SkyUiResponse] Received response for unknown or timed-out UI request ID: {}",
			request_id
		);
	}
	Ok(())
}
