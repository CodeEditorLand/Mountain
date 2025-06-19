//! # UserInterfaceProvider Implementation
//!
//! Implements the `UserInterfaceProvider` trait for the `MountainEnvironment`.
//! This provider orchestrates all modal UI interactions like dialogs, messages,
//! and quick picks by communicating with the `Sky` frontend.

use std::path::PathBuf;

use Common::{
	Error::CommonError::CommonError,
	UserInterface::{
		DTO::{
			InputBoxOptionsDTO::InputBoxOptionsDTO,
			MessageSeverity::MessageSeverity,
			OpenDialogOptionsDTO::OpenDialogOptionsDTO,
			QuickPickItemDTO::QuickPickItemDTO,
			QuickPickOptionsDTO::QuickPickOptionsDTO,
			SaveDialogOptionsDTO::SaveDialogOptionsDTO,
		},
		UserInterfaceProvider::UserInterfaceProvider,
	},
};
use async_trait::async_trait;
use log::{info, warn};
use serde::Serialize;
use serde_json::{Value, json};
use tauri::Emitter;
use tokio::time::{Duration, timeout};
use uuid::Uuid;

use super::{MountainEnvironment::MountainEnvironment, Utility};

#[derive(Serialize, Clone)]
struct UserInterfaceRequest<TPayload:Serialize + Clone> {
	pub RequestIdentifier:String,
	pub Payload:TPayload,
}

#[async_trait]
impl UserInterfaceProvider for MountainEnvironment {
	/// Shows a message to the user with a given severity and optional action
	/// buttons.
	async fn ShowMessage(
		&self,
		Severity:MessageSeverity,
		Message:String,
		Options:Option<Value>,
	) -> Result<Option<String>, CommonError> {
		info!("[UserInterfaceProvider] Showing interactive message: {}", Message);
		let Payload = json!({ "Severity": Severity, "Message": Message, "Options": Options });
		let ResponseValue = SendUserInterfaceRequest(self, "sky://ui/show-message-request", Payload).await?;
		Ok(ResponseValue.as_str().map(String::from))
	}

	/// Shows a dialog for opening files or folders.
	async fn ShowOpenDialog(&self, Options:Option<OpenDialogOptionsDTO>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		info!("[UserInterfaceProvider] Showing open dialog.");
		let ResponseValue = SendUserInterfaceRequest(self, "sky://ui/show-open-dialog-request", Options).await?;
		serde_json::from_value(ResponseValue).map_err(|e| {
			CommonError::SerializationError { Description:format!("Failed to deserialize open dialog response: {}", e) }
		})
	}

	/// Shows a dialog for saving a file.
	async fn ShowSaveDialog(&self, Options:Option<SaveDialogOptionsDTO>) -> Result<Option<PathBuf>, CommonError> {
		info!("[UserInterfaceProvider] Showing save dialog.");
		let ResponseValue = SendUserInterfaceRequest(self, "sky://ui/show-save-dialog-request", Options).await?;
		serde_json::from_value(ResponseValue).map_err(|e| {
			CommonError::SerializationError { Description:format!("Failed to deserialize save dialog response: {}", e) }
		})
	}

	/// Shows a quick pick list to the user.
	async fn ShowQuickPick(
		&self,
		Items:Vec<QuickPickItemDTO>,
		Options:Option<QuickPickOptionsDTO>,
	) -> Result<Option<Vec<String>>, CommonError> {
		info!("[UserInterfaceProvider] Showing quick pick with {} items.", Items.len());
		let Payload = json!({ "Items": Items, "Options": Options });
		let ResponseValue = SendUserInterfaceRequest(self, "sky://ui/show-quick-pick-request", Payload).await?;
		serde_json::from_value(ResponseValue).map_err(|e| {
			CommonError::SerializationError { Description:format!("Failed to deserialize quick pick response: {}", e) }
		})
	}

	/// Shows an input box to solicit a string input from the user.
	async fn ShowInputBox(&self, Options:Option<InputBoxOptionsDTO>) -> Result<Option<String>, CommonError> {
		info!("[UserInterfaceProvider] Showing input box.");
		let ResponseValue = SendUserInterfaceRequest(self, "sky://ui/show-input-box-request", Options).await?;
		serde_json::from_value(ResponseValue).map_err(|e| {
			CommonError::SerializationError { Description:format!("Failed to deserialize input box response: {}", e) }
		})
	}
}

// --- Internal Helper Functions ---

/// A generic helper function to send a request to the Sky UI and wait for a
/// response.
///
/// This function implements a robust request-response pattern over Tauri's
/// event system using `tokio::sync::oneshot` channels for communication.
///
/// # Parameters
/// * `Environment`: The `MountainEnvironment` instance.
/// * `EventName`: The name of the event to emit to the Sky frontend.
/// * `Payload`: The serializable data to send with the event.
///
/// # Returns
/// A `Result` containing the `serde_json::Value` response from the UI.
async fn SendUserInterfaceRequest<TPayload:Serialize + Clone>(
	Environment:&MountainEnvironment,
	EventName:&str,
	Payload:TPayload,
) -> Result<Value, CommonError> {
	let RequestIdentifier = Uuid::new_v4().to_string();
	let (Sender, Receiver) = tokio::sync::oneshot::channel();

	// Store the sender half of the channel so the response handler can resolve it.
	{
		let mut PendingRequestsGuard = Environment
			.ApplicationState
			.PendingUserInterfaceRequests
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
		PendingRequestsGuard.insert(RequestIdentifier.clone(), Sender);
	}

	let EventPayload = UserInterfaceRequest { RequestIdentifier:RequestIdentifier.clone(), Payload };

	// Emit the event to the frontend.
	Environment.ApplicationHandle.emit(EventName, EventPayload).map_err(|e| {
		CommonError::UserInterfaceInteraction {
			Reason:format!("Failed to emit UI request '{}': {}", EventName, e.to_string()),
		}
	})?;

	// Wait for the response with a generous timeout for user interaction.
	match timeout(Duration::from_secs(300), Receiver).await {
		Ok(Ok(Ok(Value))) => Ok(Value),
		Ok(Ok(Err(Error))) => Err(Error),
		Ok(Err(_)) => {
			Err(CommonError::UserInterfaceInteraction {
				Reason:format!("UI response channel closed for request ID: {}", RequestIdentifier),
			})
		},
		Err(_) => {
			warn!(
				"[UserInterfaceProvider] UI request '{}' with ID {} timed out.",
				EventName, RequestIdentifier
			);
			// Clean up the stale request from the map.
			Environment
				.ApplicationState
				.PendingUserInterfaceRequests
				.lock()
				.unwrap()
				.remove(&RequestIdentifier);
			Err(CommonError::UserInterfaceInteraction {
				Reason:format!("UI request timed out for request ID: {}", RequestIdentifier),
			})
		},
	}
}
