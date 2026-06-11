//! `workspaces:enterWorkspace` IPC handler.
//!
//! VS Code passes `{ uri: { scheme, path, ... } }` or a bare string path
//! as the first argument. Extract the file-system path, read the
//! `.code-workspace` JSON, then replace the current workspace folders
//! and emit a `sky://workspace/enter` event so the workbench reloads its
//! sidebar and title. Relative folder paths resolve against the
//! directory containing the `.code-workspace` file.

use std::sync::Arc;

use serde_json::Value;
use tauri::AppHandle;

use crate::{
	ApplicationState::{
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		State::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndBroadcast,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub async fn Fn(
	ApplicationHandle:AppHandle,

	RunTime:Arc<ApplicationRunTime>,

	Arguments:Vec<Value>,
) -> Result<Value, String> {
	dev_log!("workspaces", "workspaces:enterWorkspace");

	let RawArg = Arguments.first().cloned().unwrap_or(Value::Null);

	let WorkspacePath = if let Some(UriObj) = RawArg.as_object() {
		// Serialised `vscode.Uri` - prefer the `path` field.
		UriObj.get("path").and_then(|V| V.as_str()).map(str::to_string).or_else(|| {
			// Sometimes the whole thing is `{ _formatted: "file:///..." }`
			UriObj
				.get("_formatted")
				.and_then(|V| V.as_str())
				.and_then(|S| url::Url::parse(S).ok())
				.and_then(|U| U.to_file_path().ok())
				.map(|P| P.to_string_lossy().into_owned())
		})
	} else {
		// Plain string argument - may be a `file://…` URI or a raw POSIX
		// path.
		RawArg.as_str().map(|S| {
			if S.starts_with("file://") {
				url::Url::parse(S)
					.ok()
					.and_then(|U| U.to_file_path().ok())
					.map(|P| P.to_string_lossy().into_owned())
					.unwrap_or_else(|| S.to_string())
			} else {
				S.to_string()
			}
		})
	};

	if let Some(FilePath) = WorkspacePath {
		let FilePathBuf = std::path::PathBuf::from(&FilePath);

		match tokio::fs::read_to_string(&FilePathBuf).await {
			Ok(Contents) => {
				let Parsed:Result<serde_json::Value, _> = serde_json::from_str(&Contents);

				match Parsed {
					Ok(Workspace) => {
						let Folders:Vec<WorkspaceFolderStateDTO> = Workspace
							.get("folders")
							.and_then(|V| V.as_array())
							.map(|Array| {
								Array
									.iter()
									.enumerate()
									.filter_map(|(Index, Entry)| {
										let FolderPath = Entry.get("path").and_then(|V| V.as_str())?;

										// Resolve relative paths against the
										// directory that contains the
										// .code-workspace file.
										let Resolved = if std::path::Path::new(FolderPath).is_absolute() {
											std::path::PathBuf::from(FolderPath)
										} else {
											FilePathBuf
												.parent()
												.unwrap_or_else(|| std::path::Path::new("/"))
												.join(FolderPath)
										};

										let UriStr = format!("file://{}", Resolved.to_string_lossy());

										let Uri = url::Url::parse(&UriStr).ok()?;

										let Name =
											Entry.get("name").and_then(|V| V.as_str()).unwrap_or("").to_string();

										WorkspaceFolderStateDTO::New(Uri, Name, Index).ok()
									})
									.collect()
							})
							.unwrap_or_default();

						let State = &RunTime.Environment.ApplicationState;

						UpdateWorkspaceFoldersAndBroadcast(&ApplicationHandle, &State.Workspace, Folders);

						// Signal the workbench to reload its workspace
						// context (sidebar tree, title bar, breadcrumb).
						use tauri::Emitter;

						if let Err(Error) =
							ApplicationHandle.emit("sky://workspace/enter", serde_json::json!({ "uri": FilePath }))
						{
							dev_log!("workspaces", "warn: [enterWorkspace] sky emit failed: {}", Error);
						}

						dev_log!("workspaces", "[enterWorkspace] loaded workspace from {}", FilePath);
					},

					Err(Error) => {
						dev_log!("workspaces", "warn: [enterWorkspace] JSON parse failed for {}: {}", FilePath, Error);
					},
				}
			},

			Err(Error) => {
				dev_log!("workspaces", "warn: [enterWorkspace] read failed for {}: {}", FilePath, Error);
			},
		}
	} else {
		dev_log!("workspaces", "warn: [enterWorkspace] no path in arguments");
	}

	Ok(Value::Null)
}
