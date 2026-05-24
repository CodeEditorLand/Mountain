//! `WorkspaceFolderCommand::MountainWorkspaceListFolders`

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

/// Return the current workspace folder set without mutating anything.
#[tauri::command]
pub async fn Fn(
	state:State<'_, Arc<ApplicationState>>,
) -> Result<Vec<WorkspaceFolderPayload>, String> {
	Ok(state
		.Workspace
		.GetWorkspaceFolders()
		.iter()
		.map(WorkspaceFolderPayload::from)
		.collect())
}
