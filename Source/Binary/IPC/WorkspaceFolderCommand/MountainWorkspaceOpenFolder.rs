//! `WorkspaceFolderCommand::MountainWorkspaceOpenFolder`

use std::{path::PathBuf, sync::Arc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, State};
use url::Url;
use crate::{
	ApplicationState::{
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		Struct::{
			ApplicationState::ApplicationState,
			WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndBroadcast,
		},
	},
	dev_log,
};

/// Open one or more workspace folders, replacing any currently-open set.
///
/// Returns the new folder list so Sky can update its sidebar without a
/// round-trip. The caller should pass absolute filesystem paths; URLs
/// are accepted and parsed, but Tauri dialog results are typically paths.
#[tauri::command]
pub async fn Fn(
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
