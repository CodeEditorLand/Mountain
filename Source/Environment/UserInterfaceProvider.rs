// File: Mountain/Source/Environment/UserInterfaceProvider.rs
// Role: Implements the `UserInterfaceProvider` trait for the
// `MountainEnvironment`. Responsibilities:
//   - Orchestrate all modal UI interactions (dialogs, messages, quick picks).
//   - Use the `tauri-plugin-dialog` for native file dialogs.
//   - Use a custom request-response event pattern for web-based UI elements.

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
use tauri_plugin_dialog::{DialogExt, FilePath};
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

	/// Shows a dialog for opening files or folders using the
	/// tauri-plugin-dialog.
	async fn ShowOpenDialog(&self, Options:Option<OpenDialogOptionsDTO>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		info!("[UserInterfaceProvider] Showing open dialog.");
		let mut builder = self.ApplicationHandle.dialog().file();

		let (can_select_many, can_select_folders, can_select_files) = if let Some(ref opts) = Options {
			// Set common options
			if let Some(title) = &opts.Base.Title {
				builder = builder.set_title(title);
			}
			if let Some(path_string) = &opts.Base.DefaultPath {
				builder = builder.set_directory(PathBuf::from(path_string));
			}
			if let Some(filters) = &opts.Base.FilterList {
				for filter in filters {
					let extensions:Vec<&str> = filter.ExtensionList.iter().map(AsRef::as_ref).collect();
					builder = builder.add_filter(&filter.Name, &extensions);
				}
			}
			(
				opts.CanSelectMany.unwrap_or(false),
				opts.CanSelectFolders.unwrap_or(false),
				opts.CanSelectFiles.unwrap_or(true), // Default to true if not specified
			)
		} else {
			(false, false, true)
		};

		// Spawn blocking task to avoid blocking async runtime
		let picked_paths:Option<Vec<FilePath>> = tokio::task::spawn_blocking(move || {
			if can_select_folders {
				if can_select_many {
					builder.blocking_pick_folders()
				} else {
					builder.blocking_pick_folder().map(|p| vec![p])
				}
			} else if can_select_files {
				if can_select_many {
					builder.blocking_pick_files()
				} else {
					builder.blocking_pick_file().map(|p| vec![p])
				}
			} else {
				None
			}
		})
		.await
		.map_err(|e| CommonError::UserInterfaceInteraction { Reason:format!("Dialog task failed: {}", e) })?;

		// Convert the result from the dialog's FilePath type to standard PathBuf
		let result = picked_paths.map(|file_paths| file_paths.into_iter().filter_map(|p| p.into_path().ok()).collect());

		Ok(result)
	}

	/// Shows a dialog for saving a file using the tauri-plugin-dialog.
	async fn ShowSaveDialog(&self, Options:Option<SaveDialogOptionsDTO>) -> Result<Option<PathBuf>, CommonError> {
		info!("[UserInterfaceProvider] Showing save dialog.");

		let mut builder = self.ApplicationHandle.dialog().file();

		if let Some(options) = Options {
			if let Some(title) = options.Base.Title {
				builder = builder.set_title(title);
			}
			if let Some(path_string) = options.Base.DefaultPath {
				let path = PathBuf::from(path_string);
				// If a parent directory exists, set it.
				if let Some(parent) = path.parent() {
					builder = builder.set_directory(parent);
				}
				// If a file name exists, set it as the default name.
				if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
					builder = builder.set_file_name(file_name);
				}
			}
			if let Some(filters) = options.Base.FilterList {
				for filter in filters {
					let extensions:Vec<&str> = filter.ExtensionList.iter().map(AsRef::as_ref).collect();
					builder = builder.add_filter(filter.Name, &extensions);
				}
			}
		}

		let picked_file = tokio::task::spawn_blocking(move || builder.blocking_save_file())
			.await
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:format!("Dialog task failed: {}", e) })?;

		// Convert the result to a standard PathBuf
		let result = picked_file.and_then(|p| p.into_path().ok());

		Ok(result)
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
async fn SendUserInterfaceRequest<TPayload:Serialize + Clone>(
	Environment:&MountainEnvironment,
	EventName:&str,
	Payload:TPayload,
) -> Result<Value, CommonError> {
	let RequestIdentifier = Uuid::new_v4().to_string();
	let (Sender, Receiver) = tokio::sync::oneshot::channel();

	{
		let mut PendingRequestsGuard = Environment
			.ApplicationState
			.PendingUserInterfaceRequests
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
		PendingRequestsGuard.insert(RequestIdentifier.clone(), Sender);
	}

	let EventPayload = UserInterfaceRequest { RequestIdentifier:RequestIdentifier.clone(), Payload };

	Environment.ApplicationHandle.emit(EventName, EventPayload).map_err(|e| {
		CommonError::UserInterfaceInteraction {
			Reason:format!("Failed to emit UI request '{}': {}", EventName, e.to_string()),
		}
	})?;

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
