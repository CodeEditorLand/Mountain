// @module UiLogic
// @description Contains the core logic for orchestrating User Interface interactions like
// dialogs, messages, and quick picks by communicating with the Sky frontend.

use std::path::PathBuf;

use Common::{
	error::CommonError,
	ui::DTO::{
		InputBoxOptionsDTO,
		MessageSeverity,
		OpenDialogOptionsDTO,
		QuickPickItemDTO,
		QuickPickOptionsDTO,
		SaveDialogOptionsDTO,
	},
};
use log::{info, warn};
use serde::Serialize;
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tokio::time::{Duration, timeout};
use uuid::Uuid;

use crate::ApplicationState::ApplicationState::ApplicationState;

#[derive(Serialize, Clone)]
struct UiRequestPayload<T:Serialize + Clone> {
	pub RequestIdentifier:String,
	pub Payload:T,
}

// A generic helper function to send a request to the Sky User Interface and wait for a
// response.
//
// This function implements a robust request-response pattern over Tauri's
// event system using `tokio::sync::oneshot` channels.
//
// @param app_handle - The Tauri application handle.
// @param event_name - The name of the event to emit to the Sky frontend.
// @param payload - The serializable data to send with the event.
// @returns A `Result` containing the `serde_json::Value` response from the User Interface.
async fn send_ui_request<P:Serialize + Clone, R:tauri::Runtime>(
	app_handle:&AppHandle<R>,
	event_name:&str,
	payload:P,
) -> Result<Value, CommonError> {
	let request_id = Uuid::new_v4().to_string();
	let (tx, rx) = tokio::sync::oneshot::channel();

	// Store the sender half of the channel so the response handler can resolve it.
	{
		let app_state = app_handle.state::<ApplicationState>();
		let mut pending_requests_guard = app_state.PendingUiRequests.lock().unwrap();
		pending_requests_guard.insert(request_id.clone(), tx);
	}

	let event_payload = UiRequestPayload { RequestIdentifier:request_id.clone(), Payload:payload };

	// Emit the event to the frontend.
	app_handle.emit(event_name, event_payload).map_err(|e| {
		CommonError::UiInteraction { Reason:format!("Failed to emit User Interface request '{}': {}", event_name, e) }
	})?;

	// Wait for the response with a generous timeout for user interaction.
	match timeout(Duration::from_secs(300), rx).await {
		Ok(Ok(Ok(value))) => Ok(value),
		Ok(Ok(Err(error))) => Err(error),
		Ok(Err(_)) => {
			Err(CommonError::UiInteraction {
				Reason:format!("User Interface response channel closed for request ID: {}", request_id),
			})
		},
		Err(_) => {
			warn!("[UiLogic] User Interface request '{}' with ID {} timed out.", event_name, request_id);
			// Clean up the stale request from the map.
			let app_state = app_handle.state::<ApplicationState>();
			app_state.PendingUiRequests.lock().unwrap().remove(&request_id);
			Err(CommonError::UiInteraction { Reason:format!("User Interface request timed out for request ID: {}", request_id) })
		},
	}
}

// Logic to show a message to the user.
pub async fn ShowMessageInteractiveLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	severity:MessageSeverity,
	message:String,
	options:Option<Value>,
) -> Result<Option<String>, CommonError> {
	info!("[UiLogic] Showing interactive message: {}", message);
	let payload = json!({ "Severity": severity, "Message": message, "Options": options });
	let response_value = send_ui_request(app_handle, "sky://ui/show-message-request", payload).await?;
	Ok(response_value.as_str().map(String::from))
}

// Logic to show a native file open dialog.
pub async fn ShowOpenDialogInteractiveLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	options:Option<OpenDialogOptionsDTO>,
) -> Result<Option<Vec<PathBuf>>, CommonError> {
	info!("[UiLogic] Showing open dialog.");
	let response_value = send_ui_request(app_handle, "sky://ui/show-open-dialog-request", options).await?;
	serde_json::from_value(response_value).map_err(|e| {
		CommonError::SerdeError { Description:format!("Failed to deserialize open dialog response: {}", e) }
	})
}

// Logic to show a native file save dialog.
pub async fn ShowSaveDialogInteractiveLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	options:Option<SaveDialogOptionsDTO>,
) -> Result<Option<PathBuf>, CommonError> {
	info!("[UiLogic] Showing save dialog.");
	let response_value = send_ui_request(app_handle, "sky://ui/show-save-dialog-request", options).await?;
	serde_json::from_value(response_value).map_err(|e| {
		CommonError::SerdeError { Description:format!("Failed to deserialize save dialog response: {}", e) }
	})
}

// Logic to show a quick pick list to the user.
pub async fn ShowQuickPickInteractiveLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	items:Vec<QuickPickItemDTO>,
	options:Option<QuickPickOptionsDTO>,
) -> Result<Option<Vec<String>>, CommonError> {
	info!("[UiLogic] Showing quick pick with {} items.", items.len());
	let payload = json!({ "Items": items, "Options": options });
	let response_value = send_ui_request(app_handle, "sky://ui/show-quick-pick-request", payload).await?;
	serde_json::from_value(response_value).map_err(|e| {
		CommonError::SerdeError { Description:format!("Failed to deserialize quick pick response: {}", e) }
	})
}

// Logic to show an input box to the user.
pub async fn ShowInputBoxInteractiveLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	options:Option<InputBoxOptionsDTO>,
) -> Result<Option<String>, CommonError> {
	info!("[UiLogic] Showing input box.");
	let response_value = send_ui_request(app_handle, "sky://ui/show-input-box-request", options).await?;
	serde_json::from_value(response_value)
		.map_err(|e| CommonError::SerdeError { Description:format!("Failed to deserialize input box response: {}", e) })
}
