//! `WorkspaceFolderCommand::MountainWorkspaceCloseAllFolders`

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

/// Close every workspace folder - equivalent to VS Code's
/// `workbench.Action.closeFolder`. Extensions that subscribe to
/// `onDidChangeWorkspaceFolders` receive an event whose `removed` array
/// contains every previously-open folder.
#[tauri::command]
pub async fn Fn(
	app_handle:AppHandle,

	state:State<'_, Arc<ApplicationState>>,
) -> Result<Value, String> {
	UpdateWorkspaceFoldersAndBroadcast(&app_handle, &state.Workspace, Vec::new());

	dev_log!("lifecycle", "[WorkspaceFolderCommand] All folders closed");

	Ok(Value::Null)
}
