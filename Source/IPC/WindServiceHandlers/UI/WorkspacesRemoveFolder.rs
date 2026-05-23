#![allow(unused_variables)]

//! Wire method: `workspaces:removeFolder`.

use std::sync::Arc;

use serde_json::Value;

use crate::{
	ApplicationState::State::WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndNotify,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
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
