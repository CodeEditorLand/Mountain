#![allow(non_snake_case)]

//! Workspace domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{
	ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// Return the current workspace folders.
pub async fn handle_workspaces_get_folders(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Workspace = &Runtime.Environment.ApplicationState.Workspace;
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

/// Add a workspace folder.
pub async fn handle_workspaces_add_folder(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	use url::Url;

	let UriStr = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workspaces:addFolder requires uri as first argument".to_string())?
		.to_string();

	let Name = Args.get(1).and_then(|V| V.as_str()).unwrap_or("").to_string();

	let Workspace = &Runtime.Environment.ApplicationState.Workspace;
	let mut Folders = Workspace.GetWorkspaceFolders();
	let Index = Folders.len();
	let URI = Url::parse(&UriStr).map_err(|E| format!("workspaces:addFolder invalid URI: {}", E))?;
	if let Ok(Folder) = WorkspaceFolderStateDTO::New(URI, Name, Index) {
		Folders.push(Folder);
		Workspace.SetWorkspaceFolders(Folders);
	}

	Ok(Value::Null)
}

/// Remove a workspace folder by URI.
pub async fn handle_workspaces_remove_folder(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let UriStr = Args
		.first()
		.and_then(|V| V.as_str())
		.ok_or("workspaces:removeFolder requires uri as first argument".to_string())?
		.to_string();

	let Workspace = &Runtime.Environment.ApplicationState.Workspace;
	let mut Folders = Workspace.GetWorkspaceFolders();
	Folders.retain(|F| F.URI.to_string() != UriStr);
	for (I, F) in Folders.iter_mut().enumerate() {
		F.Index = I;
	}
	Workspace.SetWorkspaceFolders(Folders);

	Ok(Value::Null)
}

/// Return the workspace name (basename of root folder, or None if untitled).
pub async fn handle_workspaces_get_name(Runtime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Name = Runtime
		.Environment
		.ApplicationState
		.Workspace
		.GetWorkspaceFolders()
		.into_iter()
		.next()
		.map(|F| F.GetDisplayName());

	Ok(Name.map(|N| json!(N)).unwrap_or(Value::Null))
}
