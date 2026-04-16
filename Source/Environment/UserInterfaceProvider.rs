//! # UserInterfaceProvider (Environment)
//!
//! Implements the `UserInterfaceProvider` trait for `MountainEnvironment`,
//! orchestrating all modal UI interactions like dialogs, messages, and quick
//! picks by communicating with the `Sky` frontend.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Modal Dialogs
//! - Open file/folder selection dialogs (`OpenDialog`)
//! - Save file dialogs (`SaveDialog`)
//! - Message boxes (`ShowMessage`, `ShowErrorMessage`)
//! - Input boxes for text entry (`InputBox`)
//! - Quick pick lists for selection (`QuickPick`)
//!
//! ### 2. Request-Response Pattern
//! - Send UI requests to Sky frontend via IPC
//! - Track pending requests with unique IDs
//! - Wait for responses with timeout handling
//! - Resolve results via `ResolveUIRequest` callback
//!
//! ### 3. Thread Safety
//! - All methods are async and safe for concurrent access
//! - Pending requests stored in
//! `ApplicationState.UI.PendingUserInterfaceRequest`
//! - Uses `tokio::sync::oneshot` for request-response coordination
//!
//! ## ARCHITECTURAL ROLE
//!
//! UserInterfaceProvider is the **UI bridge** for Mountain:
//!
//! ```text
//! Provider ──► UI Request ──► Sky Frontend ──► User Interaction ──► ResolveUIRequest
//! ```
//!
//! ### Position in Mountain
//! - `Environment` module: UI capability provider
//! - Implements `CommonLibrary::UserInterface::UserInterfaceProvider` trait
//! - Accessible via `Environment.Require<dyn UserInterfaceProvider>()`
//!
//! ### Dependencies
//! - `ApplicationState`: Pending request tracking
//! - `IPCProvider`: For sending messages to Sky
//! - `tauri::AppHandle`: For window/parent references
//!
//! ### Dependents
//! - Any command that needs to show UI dialogs
//! - `DispatchLogic::ResolveUIRequest`: Completes the request-response cycle
//! - Error handlers: Show error messages to users
//!
//! ## DTO STRUCTURES
//!
//! All UI operations use DTOs for type-safe options:
//! - `OpenDialogOptionsDTO`: File/folder selection options
//! - `SaveDialogOptionsDTO`: Save file dialog options
//! - `QuickPickOptionsDTO`: Quick pick list configuration
//! - `InputBoxOptionsDTO`: Input box configuration
//! - `MessageSeverity`: Info, Warning, Error levels
//!
//! ## REQUEST FLOW
//!
//! 1. Provider method called (e.g., `ShowMessage`)
//! 2. Generate unique request ID
//! 3. Store `oneshot::Sender` in `PendingUserInterfaceRequest` map
//! 4. Send IPC message to Sky with request ID and options
//! 5. Sky shows UI and waits for user action
//! 6. User responds → Sky calls `ResolveUIRequest` Tauri command
//! 7. `ResolveUIRequest` looks up sender by ID and sends result
//! 8. Provider method returns result to caller
//!
//! ## ERROR HANDLING
//!
//! - IPC failures: `CommonError::IPCError`
//! - Timeout: `CommonError::RequestTimeout`
//! - User cancellation: `None` result (not error)
//! - Invalid arguments: `CommonError::InvalidArgument`
//!
//! ## PERFORMANCE
//!
//! - Requests are async and non-blocking
//! - Timeouts prevent indefinite waiting (default ~30s)
//! - Request IDs are time-based for uniqueness
//! - Pending request map uses `Arc<Mutex<>>` for thread safety
//!
//! ## VS CODE REFERENCE
//!
//! Borrowed from VS Code's UI system:
//! - `vs/platform/dialogs/common/dialogs.ts` - Dialog service API
//! - `vs/platform/prompt/common/prompt.ts` - Input and quick pick
//! - `vs/workbench/services/decorator/common/decorator.ts` - Message service
//!
//! ## TODO
//!
//! - [ ] Add support for custom dialog buttons and layouts
//! - [ ] Implement file/folder filters with glob patterns
//! - [ ] Add dialog position and sizing controls
//! - [ ] Support modal vs non-modal dialogs
//! - [ ] Add accessibility features (screen reader support)
//! - [ ] Implement dialog theming (dark/light mode)
//! - [ ] Add file type/extension selection in save dialog
//! - [ ] Support multi-select in quick pick and file dialogs
//! - [ ] Add async progress reporting during long operations
//! - [ ] Implement custom input validation (regex, etc.)
//!
//! ## MODULE CONTENTS
//!
//! - [`UserInterfaceProvider`]: Main struct implementing the trait
//! - Dialog-specific methods: `ShowMessage`, `OpenDialog`, `SaveDialog`
//! - Selection methods: `QuickPick`, `InputBox`
//! - Request-response coordination logic

use std::path::PathBuf;

use CommonLibrary::{
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
use serde::Serialize;
use serde_json::{Value, json};
use tauri::Emitter;
use tauri_plugin_dialog::{DialogExt, FilePath};
use tokio::time::{Duration, timeout};
use uuid::Uuid;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::dev_log;

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
		dev_log!("window", "[UserInterfaceProvider] Showing interactive message: {}", Message);

		let Payload = json!({ "Severity": Severity, "Message": Message, "Options": Options });

		let ResponseValue = SendUserInterfaceRequest(self, "sky://ui/show-message-request", Payload).await?;

		Ok(ResponseValue.as_str().map(String::from))
	}

	/// Shows a dialog for opening files or folders using the
	/// tauri-plugin-dialog.
	async fn ShowOpenDialog(&self, Options:Option<OpenDialogOptionsDTO>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		dev_log!("window", "[UserInterfaceProvider] Showing open dialog.");

		let mut Builder = self.ApplicationHandle.dialog().file();

		let (CanSelectMany, CanSelectFolders, CanSelectFiles) = if let Some(ref opts) = Options {
			if let Some(title) = &opts.Base.Title {
				Builder = Builder.set_title(title);
			}

			if let Some(path_string) = &opts.Base.DefaultPath {
				Builder = Builder.set_directory(PathBuf::from(path_string));
			}

			if let Some(filters) = &opts.Base.FilterList {
				for filter in filters {
					let extensions:Vec<&str> = filter.ExtensionList.iter().map(AsRef::as_ref).collect();

					Builder = Builder.add_filter(&filter.Name, &extensions);
				}
			}

			(
				opts.CanSelectMany.unwrap_or(false),
				opts.CanSelectFolders.unwrap_or(false),
				opts.CanSelectFiles.unwrap_or(true),
			)
		} else {
			(false, false, true)
		};

		let PickedPaths:Option<Vec<FilePath>> = tokio::task::spawn_blocking(move || {
			if CanSelectFolders {
				if CanSelectMany {
					Builder.blocking_pick_folders()
				} else {
					Builder.blocking_pick_folder().map(|p| vec![p])
				}
			} else if CanSelectFiles {
				if CanSelectMany {
					Builder.blocking_pick_files()
				} else {
					Builder.blocking_pick_file().map(|p| vec![p])
				}
			} else {
				None
			}
		})
		.await
		.map_err(|Error| CommonError::UserInterfaceInteraction { Reason:format!("Dialog task failed: {}", Error) })?;

		Ok(PickedPaths.map(|paths| paths.into_iter().filter_map(|p| p.into_path().ok()).collect()))
	}

	/// Shows a dialog for saving a file using the tauri-plugin-dialog.
	async fn ShowSaveDialog(&self, Options:Option<SaveDialogOptionsDTO>) -> Result<Option<PathBuf>, CommonError> {
		dev_log!("window", "[UserInterfaceProvider] Showing save dialog.");

		let mut Builder = self.ApplicationHandle.dialog().file();

		if let Some(options) = Options {
			if let Some(title) = options.Base.Title {
				Builder = Builder.set_title(title);
			}

			if let Some(path_string) = options.Base.DefaultPath {
				let path = PathBuf::from(path_string);

				if let Some(parent) = path.parent() {
					Builder = Builder.set_directory(parent);
				}

				if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
					Builder = Builder.set_file_name(file_name);
				}
			}

			if let Some(filters) = options.Base.FilterList {
				for filter in filters {
					let extensions:Vec<&str> = filter.ExtensionList.iter().map(AsRef::as_ref).collect();

					Builder = Builder.add_filter(filter.Name, &extensions);
				}
			}
		}

		let PickedFile = tokio::task::spawn_blocking(move || Builder.blocking_save_file())
			.await
			.map_err(|Error| {
				CommonError::UserInterfaceInteraction { Reason:format!("Dialog task failed: {}", Error) }
			})?;

		Ok(PickedFile.and_then(|p| p.into_path().ok()))
	}

	/// Shows a quick pick list to the user.
	async fn ShowQuickPick(
		&self,

		Items:Vec<QuickPickItemDTO>,

		Options:Option<QuickPickOptionsDTO>,
	) -> Result<Option<Vec<String>>, CommonError> {
		dev_log!("window", "[UserInterfaceProvider] Showing quick pick with {} items.", Items.len());

		let Payload = json!({ "Items": Items, "Options": Options });

		let ResponseValue = SendUserInterfaceRequest(self, "sky://ui/show-quick-pick-request", Payload).await?;

		serde_json::from_value(ResponseValue).map_err(|Error| {
			CommonError::SerializationError {
				Description:format!("Failed to deserialize quick pick response: {}", Error),
			}
		})
	}

	/// Shows an input box to solicit a string input from the user.
	async fn ShowInputBox(&self, Options:Option<InputBoxOptionsDTO>) -> Result<Option<String>, CommonError> {
		dev_log!("window", "[UserInterfaceProvider] Showing input box.");

		let ResponseValue = SendUserInterfaceRequest(self, "sky://ui/show-input-box-request", Options).await?;

		serde_json::from_value(ResponseValue).map_err(|Error| {
			CommonError::SerializationError {
				Description:format!("Failed to deserialize input box response: {}", Error),
			}
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
			.UI
			.PendingUserInterfaceRequest
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		PendingRequestsGuard.insert(RequestIdentifier.clone(), Sender);
	}

	let EventPayload = UserInterfaceRequest { RequestIdentifier:RequestIdentifier.clone(), Payload };

	Environment.ApplicationHandle.emit(EventName, EventPayload).map_err(|Error| {
		CommonError::UserInterfaceInteraction {
			Reason:format!("Failed to emit UI request '{}': {}", EventName, Error.to_string()),
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
			dev_log!("window", "warn: [UserInterfaceProvider] UI request '{}' with ID {} timed out.",
				EventName, RequestIdentifier);

			let mut Guard = Environment
				.ApplicationState
				.UI
				.PendingUserInterfaceRequest
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

			Guard.remove(&RequestIdentifier);

			Err(CommonError::UserInterfaceInteraction {
				Reason:format!("UI request timed out for request ID: {}", RequestIdentifier),
			})
		},
	}
}
