use Common::error::CommonError;
use log::{info, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime, command};

/// @module SkyUiResponsesLogic
/// @description Contains the handler logic for processing asynchronous
/// responses sent from the Sky frontend back to the Mountain backend. This is
/// the receiving end of the request-response pattern for UI interactions.
use crate::{AppState::AppState::AppState, handlers::error_utils};

/// A Tauri command that resolves a pending UI request. The Sky frontend calls
/// this command when a user has completed an action (e.g., selected a file in a
/// dialog, clicked a button on a message).
///
/// @param RequestId - The unique ID of the request being resolved.
/// @param DataValue - The success data from the UI (e.g., a file path).
/// @param ErrorDetailsValue - Error information if the UI operation failed.
#[command(rename_all = "PascalCase")]
pub async fn SkyResolvesUiRequest<R:Runtime>(
	AppHandle:AppHandle<R>,
	RequestId:String,
	DataValue:Option<Value>,
	ErrorDetailsValue:Option<Value>,
) -> Result<(), String> {
	info!("[SkyUiResponses] Resolving UI request ID: {}", RequestId);
	let AppStateInstance = AppHandle.state::<AppState>();

	// Atomically find and remove the pending request's sender from the map.
	let MaybeSender = AppStateInstance.PendingUiRequests.lock().unwrap().remove(&RequestId);

	if let Some(Sender) = MaybeSender {
		// Construct the result to send back to the awaiting task.
		let ResultToSend = match (DataValue, ErrorDetailsValue) {
			// If an error payload is present, the operation failed.
			(_, Some(ErrorValue)) => {
				Err(CommonError::UiInteraction { Reason:format!("Error from Sky UI: {:?}", ErrorValue) })
			},
			// If data is present, the operation succeeded.
			(Some(Data), None) => Ok(Data),
			// If both are absent, the user cancelled the operation (e.g., closed the dialog).
			(None, None) => Ok(Value::Null),
		};

		// Send the result through the oneshot channel.
		if Sender.send(ResultToSend).is_err() {
			// This happens if the backend task timed out before the UI responded.
			// It's a warning, not a critical error.
			warn!("[SkyUiResponses] UI request {} timed out before Sky responded.", RequestId);
		}
	} else {
		// This can happen if the request timed out and was already removed from the
		// map.
		warn!(
			"[SkyUiResponses] Received response for unknown or timed-out UI request ID: {}",
			RequestId
		);
	}
	Ok(())
}
