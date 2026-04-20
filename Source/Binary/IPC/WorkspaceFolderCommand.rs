//! # WorkspaceFolderCommand
//!
//! Tauri commands for opening, listing, and closing workspace folders at
//! runtime. These are the second half of Plan BATCH-02: the first half
//! (autoload at startup) seeds `ApplicationState.Workspace.WorkspaceFolders`
//! from CLI / env. These commands let Sky drive the same state change
//! after boot, for a welcome-screen "Open Folder" button or a recent-files
//! picker.
//!
//! ## Flow
//!
//! ```text
//! Sky clicks "Open Folder" ──invoke──> MountainWorkspaceOpenFolder
//!                                           │
//!                                           ▼
//!             ApplicationState.Workspace.SetWorkspaceFolders(...)
//!                                           │
//!                                           ├── UpdateWorkspaceFoldersRequest
//!                                           ▼        (to Cocoon via gRPC)
//!           extensions see new `vscode.workspace.workspaceFolders`
//!           and receive `onDidChangeWorkspaceFolders`.
//! ```
//!
//! The command deliberately validates the path before touching state: a
//! non-existent directory (user fat-fingered a drag, for instance) returns
//! an error and the existing folder set is untouched.

use std::{path::PathBuf, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, State};
use url::Url;

use crate::{
	ApplicationState::{
		ApplicationState,
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		State::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndBroadcast,
	},
	dev_log,
};

/// JSON-serialisable record returned to Sky for every folder in the set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFolderPayload {
	pub Uri:String,
	pub Name:String,
	pub Index:usize,
}

impl From<&WorkspaceFolderStateDTO> for WorkspaceFolderPayload {
	fn from(Dto:&WorkspaceFolderStateDTO) -> Self {
		Self {
			Uri:Dto.URI.to_string(),
			Name:Dto.Name.clone(),
			Index:Dto.Index,
		}
	}
}

/// Open one or more workspace folders, replacing any currently-open set.
///
/// Returns the new folder list so Sky can update its sidebar without a
/// round-trip. The caller should pass absolute filesystem paths; URLs
/// are accepted and parsed, but Tauri dialog results are typically paths.
#[tauri::command]
pub async fn MountainWorkspaceOpenFolder(
	app_handle:AppHandle,
	state:State<'_, Arc<ApplicationState>>,
	paths:Vec<String>,
) -> Result<Vec<WorkspaceFolderPayload>, String> {
	if paths.is_empty() {
		return Err("No paths provided".to_string());
	}

	let mut Folders:Vec<WorkspaceFolderStateDTO> = Vec::with_capacity(paths.len());
	for (Index, Raw) in paths.iter().enumerate() {
		let Uri = if Raw.starts_with("file:") {
			Url::parse(Raw).map_err(|Error| format!("Invalid file URL {}: {}", Raw, Error))?
		} else {
			let Path = PathBuf::from(Raw);
			if !Path.is_dir() {
				return Err(format!("Not a directory: {}", Path.display()));
			}
			let Canonical = Path.canonicalize().unwrap_or(Path.clone());
			Url::from_directory_path(&Canonical)
				.map_err(|()| format!("Failed to build directory URL for {}", Canonical.display()))?
		};
		let Name = PathBuf::from(Raw)
			.file_name()
			.and_then(|N| N.to_str())
			.map(str::to_string)
			.unwrap_or_else(|| Raw.clone());
		Folders.push(WorkspaceFolderStateDTO::New(Uri, Name, Index)?);
	}

	UpdateWorkspaceFoldersAndBroadcast(&app_handle, &state.Workspace, Folders.clone());
	dev_log!(
		"lifecycle",
		"[WorkspaceFolderCommand] Opened {} folder(s); first URI={}",
		Folders.len(),
		Folders.first().map(|F| F.URI.as_str()).unwrap_or("")
	);

	Ok(Folders.iter().map(WorkspaceFolderPayload::from).collect())
}

/// Return the current workspace folder set without mutating anything.
#[tauri::command]
pub async fn MountainWorkspaceListFolders(
	state:State<'_, Arc<ApplicationState>>,
) -> Result<Vec<WorkspaceFolderPayload>, String> {
	Ok(state
		.Workspace
		.GetWorkspaceFolders()
		.iter()
		.map(WorkspaceFolderPayload::from)
		.collect())
}

/// Close every workspace folder — equivalent to VS Code's
/// `workbench.action.closeFolder`. Extensions that subscribe to
/// `onDidChangeWorkspaceFolders` receive an event whose `removed` array
/// contains every previously-open folder.
#[tauri::command]
pub async fn MountainWorkspaceCloseAllFolders(
	app_handle:AppHandle,
	state:State<'_, Arc<ApplicationState>>,
) -> Result<Value, String> {
	UpdateWorkspaceFoldersAndBroadcast(&app_handle, &state.Workspace, Vec::new());
	dev_log!("lifecycle", "[WorkspaceFolderCommand] All folders closed");
	Ok(Value::Null)
}
