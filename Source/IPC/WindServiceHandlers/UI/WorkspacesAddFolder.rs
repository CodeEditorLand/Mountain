//! Wire method: `workspaces:addFolder`.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	ApplicationState::{
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		State::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndNotify,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use url::Url;

	let UriStr = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workspaces:addFolder requires uri as first argument".to_string())?
		.to_string();

	let Name = Arguments.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Workspace = &RunTime.Environment.ApplicationState.Workspace;

	let mut Folders = Workspace.GetWorkspaceFolders();

	let Index = Folders.len();

	let URI = Url::parse(&UriStr).map_err(|E| format!("workspaces:addFolder invalid URI: {}", E))?;

	if let Ok(Folder) = WorkspaceFolderStateDTO::New(URI, Name, Index) {
		Folders.push(Folder);

		UpdateWorkspaceFoldersAndNotify(Workspace, Folders);
	}

	Ok(Value::Null)
}
