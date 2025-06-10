use std::path::PathBuf;

use Common::{
	error::CommonError,
	ui::dto::{
		InputBoxOptionsDto,
		MessageSeverity,
		OpenDialogOptionsDto,
		QuickPickItemDto,
		QuickPickOptionsDto,
		SaveDialogOptionsDto,
	},
};
use log::{info, warn};
use serde::Serialize;
use serde_json::{Value, json};
use tauri::{ApplicationHandle, Emitter, Manager, RunTime};
use tokio::time::{Duration, timeout};
use uuid::Uuid;

// @module UiLogic
// @description Contains the core logic for orchestrating UI interactions like
// dialogs, messages, and quick picks by communicating with the Sky frontend.
use crate::{ApplicationState::ApplicationState::ApplicationState, Handler::error_utils};

#[derive(Serialize, Clone)]
struct UiRequestPayload<T:Serialize + Clone> {
	pub RequestIdentifier:String,
	pub Payload:T,
}

// A generic helper function to send a request to the Sky UI and wait for a
// response.
//
// This function implements a robust request-response pattern over Tauri's
// event system using `tokio::sync::oneshot` channels.
//
// @param ApplicationHandle - The Tauri application handle.
// @param EventName - The name of the event to emit to the Sky frontend.
// @param Payload - The serializable data to send with the event.
// @returns A `Result` containing the `serde_json::Value` response from the UI.
async fn SendUiRequest<P:Serialize + Clone, R:tauri::RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	EventName:&str,
	Payload:P,
) -> Result<Value, CommonError> {
	let RequestId = Uuid::new_v4().to_string();
	let (Tx, Rx) = tokio::sync::oneshot::channel();

	// Store the sender half of the channel so the response handler can resolve it.
	{
		let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
		let mut PendingRequestsGuard = AppStateInstance.PendingUiRequests.lock().unwrap();
		PendingRequestsGuard.insert(RequestId.clone(), Tx);
	}

	let EventPayload = UiRequestPayload { RequestIdentifier:RequestId.clone(), Payload };

	// Emit the event to the frontend.
	ApplicationHandle.emit(EventName, EventPayload).map_err(|e| {
		CommonError::UiInteraction { Reason:format!("Failed to emit UI request '{}': {}", EventName, e) }
	})?;

	// Wait for the response with a generous timeout for user interaction.
	match timeout(Duration::from_secs(300), Rx).await {
		Ok(Ok(Ok(Value))) => Ok(Value),
		Ok(Ok(Err(Error))) => Err(Error),
		Ok(Err(_)) => {
			Err(CommonError::UiInteraction {
				Reason:format!("UI response channel closed for request ID: {}", RequestId),
			})
		},
		Err(_) => {
			warn!("[UiLogic] UI request '{}' with ID {} timed out.", EventName, RequestId);
			// Clean up the stale request from the map.
			let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
			AppStateInstance.PendingUiRequests.lock().unwrap().remove(&RequestId);
			Err(CommonError::UiInteraction { Reason:format!("UI request timed out for request ID: {}", RequestId) })
		},
	}
}

// Logic to show a message to the user.
pub async fn ShowMessageInteractiveLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Severity:MessageSeverity,
	Message:String,
	Options:Option<Value>,
) -> Result<Option<String>, CommonError> {
	info!("[UiLogic] Showing interactive message: {}", Message);
	let Payload = json!({ "Severity": Severity, "Message": Message, "Options": Options });
	let ResponseValue = SendUiRequest(ApplicationHandle, "sky://ui/show-message-request", Payload).await?;
	Ok(ResponseValue.as_str().map(String::from))
}

// Logic to show a native file open dialog.
pub async fn ShowOpenDialogInteractiveLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Options:Option<OpenDialogOptionsDto>,
) -> Result<Option<Vec<PathBuf>>, CommonError> {
	info!("[UiLogic] Showing open dialog.");
	let ResponseValue = SendUiRequest(ApplicationHandle, "sky://ui/show-open-dialog-request", Options).await?;
	serde_json::from_value(ResponseValue).map_err(|e| {
		CommonError::SerdeError { Description:format!("Failed to deserialize open dialog response: {}", e) }
	})
}

// Logic to show a native file save dialog.
pub async fn ShowSaveDialogInteractiveLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Options:Option<SaveDialogOptionsDto>,
) -> Result<Option<PathBuf>, CommonError> {
	info!("[UiLogic] Showing save dialog.");
	let ResponseValue = SendUiRequest(ApplicationHandle, "sky://ui/show-save-dialog-request", Options).await?;
	serde_json::from_value(ResponseValue).map_err(|e| {
		CommonError::SerdeError { Description:format!("Failed to deserialize save dialog response: {}", e) }
	})
}

// Logic to show a quick pick list to the user.
pub async fn ShowQuickPickInteractiveLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Items:Vec<QuickPickItemDto>,
	Options:Option<QuickPickOptionsDto>,
) -> Result<Option<Vec<String>>, CommonError> {
	info!("[UiLogic] Showing quick pick with {} items.", Items.len());
	let Payload = json!({ "Items": Items, "Options": Options });
	let ResponseValue = SendUiRequest(ApplicationHandle, "sky://ui/show-quick-pick-request", Payload).await?;
	serde_json::from_value(ResponseValue).map_err(|e| {
		CommonError::SerdeError { Description:format!("Failed to deserialize quick pick response: {}", e) }
	})
}

// Logic to show an input box to the user.
pub async fn ShowInputBoxInteractiveLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Options:Option<InputBoxOptionsDto>,
) -> Result<Option<String>, CommonError> {
	info!("[UiLogic] Showing input box.");
	let ResponseValue = SendUiRequest(ApplicationHandle, "sky://ui/show-input-box-request", Options).await?;
	serde_json::from_value(ResponseValue)
		.map_err(|e| CommonError::SerdeError { Description:format!("Failed to deserialize input box response: {}", e) })
}
