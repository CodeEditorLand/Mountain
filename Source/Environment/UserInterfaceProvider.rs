//! # UserInterfaceProvider (Environment)
//!
//! Implements the `UserInterfaceProvider` trait for `MountainEnvironment`,
//! orchestrating all modal UI interactions (dialogs, messages, quick picks)
//! by communicating with the `Sky` frontend.
//!
//! ## Request-response pattern
//!
//! Every blocking UI operation follows the same flow:
//! 1. Generate a UUID request ID.
//! 2. Insert a `tokio::sync::oneshot::Sender` in
//!    `ApplicationState.UI.PendingUserInterfaceRequest`.
//! 3. Emit a Tauri event to Sky with the ID and payload.
//! 4. Await the oneshot (timeout: 300 s); `DispatchLogic::Fn`
//!    resolves it when the user responds.
//!
//! The shared helper `SendUserInterfaceRequest` (pub-crate) is also used by
//! effect creators (`applyEdit`, `showTextDocument`, `Task.Execute`) that need
//! the same request-ID / oneshot pattern instead of fire-and-forget emits.
//!
//! ## Operations
//!
//! - `ShowMessage` - modal message box (`Info` / `Warning` / `Error`)
//! - `ShowOpenDialog` - native file/folder picker (via `tauri-plugin-dialog`;
//!   supports multi-select, folder-only, and file-type filters)
//! - `ShowSaveDialog` - native save-file picker
//! - `ShowQuickPick` - Sky `sky://quickpick/show` (camelCase wire shape)
//! - `ShowInputBox` - Sky `sky://input-box/show` (camelCase wire shape)
//!
//! ## VS Code reference
//!
//! - `vs/platform/dialogs/common/dialogs.ts`
//! - `vs/platform/prompt/common/prompt.ts`

use std::path::PathBuf;

use CommonLibrary::{
	Error::CommonError::CommonError,
	IPC::SkyEvent::SkyEvent,
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

// TODO: custom dialog buttons/layouts, glob-pattern file-type filters,
// dialog position + sizing, modal vs non-modal, accessibility (screen reader),
// theming (dark/light auto), file-extension selection in save dialog,
// multi-select in quick pick + file dialogs, async progress reporting,
// custom input validation (regex).

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

		// camelCase wire shape per the project-wide audit. Sky's listener
		// at `SkyBridge.ts:2444` already tolerates both casings via the
		// `?? severity` fallbacks; emit camelCase as the canonical form.
		let Payload = json!({ "severity": Severity, "message": Message, "options": Options });

		let ResponseValue = SendUserInterfaceRequest(self, SkyEvent::UIShowMessageRequest.AsStr(), Payload).await?;

		Ok(ResponseValue.as_str().map(String::from))
	}

	/// Shows a dialog for opening files or folders using the
	/// tauri-plugin-dialog.
	async fn ShowOpenDialog(&self, Options:Option<OpenDialogOptionsDTO>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		dev_log!("window", "[UserInterfaceProvider] Showing open dialog.");

		let mut Builder = This.ApplicationHandle.dialog().file();

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

		let mut Builder = This.ApplicationHandle.dialog().file();

		if let Some(options) = Options {
			if let Some(title) = options.Base.Title {
				Builder = Builder.set_title(title);
			}

			if let Some(path_string) = options.Base.DefaultPath {
				let Path = PathBuf::from(path_string);

				if let Some(parent) = path.parent() {
					Builder = Builder.set_directory(parent);
				}

				if let Some(file_name) = path.file_name().and_then(|N| n.to_str()) {
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
		dev_log!(
			"window",
			"[UserInterfaceProvider] Showing quick pick with {} items.",
			Items.len()
		);

		// camelCase wire shape per project-wide audit.
		let Payload = json!({ "items": Items, "options": Options });

		// Use the Sky-listener-aligned channel (`sky://quickpick/show`).
		// The legacy `UIShowQuickPickRequest` channel
		// (`sky://ui/show-quick-pick-request`) had no Sky listener and
		// every emit silently disappeared.
		let ResponseValue = SendUserInterfaceRequest(self, SkyEvent::QuickPickShow.AsStr(), Payload).await?;

		serde_json::from_value(ResponseValue).map_err(|Error| {
			CommonError::SerializationError {
				Description:format!("Failed to deserialize quick pick response: {}", Error),
			}
		})
	}

	/// Shows an input box to solicit a string input from the user.
	async fn ShowInputBox(&self, Options:Option<InputBoxOptionsDTO>) -> Result<Option<String>, CommonError> {
		dev_log!("window", "[UserInterfaceProvider] Showing input box.");

		// Use the Sky-listener-aligned channel (`sky://input-box/show`).
		// The legacy `UIShowInputBoxRequest` channel
		// (`sky://ui/show-input-box-request`) had no Sky listener.
		let ResponseValue = SendUserInterfaceRequest(self, SkyEvent::InputBoxShow.AsStr(), Options).await?;

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
///
/// Atom T1: made `pub(crate)` so Track effect creators
/// (`applyEdit` / `showTextDocument` / `Task.Execute`, etc.) can reuse the
/// same RequestIdentifier/oneshot pattern instead of emitting fire-and-
/// forget events that resolve to synthetic success.
pub(crate) async fn SendUserInterfaceRequest<TPayload:Serialize + Clone>(
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
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

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
			dev_log!(
				"window",
				"warn: [UserInterfaceProvider] UI request '{}' with ID {} timed out.",
				EventName,
				RequestIdentifier
			);

			let mut Guard = Environment
				.ApplicationState
				.UI
				.PendingUserInterfaceRequest
				.lock()
				.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

			Guard.remove(&RequestIdentifier);

			Err(CommonError::UserInterfaceInteraction {
				Reason:format!("UI request timed out for request ID: {}", RequestIdentifier),
			})
		},
	}
}
