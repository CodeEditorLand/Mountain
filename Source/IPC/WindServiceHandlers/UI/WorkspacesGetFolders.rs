//! Wire method: `workspaces:getFolders`.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
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
