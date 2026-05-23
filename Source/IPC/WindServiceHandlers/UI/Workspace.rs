#![allow(unused_variables)]
//! Workspace folder handlers: get / add / remove / get-name. Thin
//! wrappers over `ApplicationState::Workspace` that re-broadcast via
//! `UpdateWorkspaceFoldersAndNotify` so Wind + Sky pick up the new
//! folder list on the `sky://workspace/*` channels.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{
	ApplicationState::{
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		State::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndNotify,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

pub async fn WorkspacesGetFolders(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Workspace = &RunTime.Environment.ApplicationState.Workspace;

	let Folders = Workspace.GetWorkspaceFolders();

	let FolderList:Vec<Value> = Folders
		.iter()
		.enumerate()
		.map(|(Index, Folder)| {
			json!({
				"uri": Folder.URI.to_string(),
				"name": Folder.Name,
				"index": Index,
			})
		})
		.collect();

	Ok(json!(FolderList))
}

pub async fn WorkspacesAddFolder(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
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

pub async fn WorkspacesRemoveFolder(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let UriStr = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workspaces:removeFolder requires uri as first argument".to_string())?
		.to_string();

	let Workspace = &RunTime.Environment.ApplicationState.Workspace;

	let mut Folders = Workspace.GetWorkspaceFolders();

	Folders.retain(|F| F.URI.to_string() != UriStr);

	for (I, F) in Folders.iter_mut().enumerate() {
		F.Index = I;
	}

	UpdateWorkspaceFoldersAndNotify(Workspace, Folders);

	Ok(Value::Null)
}

pub async fn WorkspacesGetName(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Name = RunTime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| F.GetDisplayName());

	Ok(Name.map(|N| json!(N)).unwrap_or(Value::Null))
}
