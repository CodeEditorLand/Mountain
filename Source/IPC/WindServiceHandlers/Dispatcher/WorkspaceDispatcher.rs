//! Workspace dispatcher.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::{
	UI::{
		WorkspacesAddFolder::Fn as WorkspacesAddFolder,
		WorkspacesGetFolders::Fn as WorkspacesGetFolders,
		WorkspacesGetName::Fn as WorkspacesGetName,
		WorkspacesRemoveFolder::Fn as WorkspacesRemoveFolder,
	},
	Utilities::{
		JsonValueHelpers::arg_string,
		RecentlyOpened::{Mutate::Fn as MutateRecentlyOpened, Read::Fn as ReadRecentlyOpened},
	},
};

/// Dispatches workspace commands.
pub async fn dispatch_workspace(
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"workspaces:getFolders" | "workspaces:getWorkspaceFolders" | "workspaces:getWorkspace" => {
			WorkspacesGetFolders(runtime.clone()).await
		},

		"workspaces:addFolder" | "workspaces:addWorkspaceFolders" => {
			WorkspacesAddFolder(runtime.clone(), arguments).await
		},

		"workspaces:removeFolder" | "workspaces:removeWorkspaceFolders" => {
			WorkspacesRemoveFolder(runtime.clone(), arguments).await
		},

		"workspaces:getName" => WorkspacesGetName(runtime.clone()).await,

		"workspaces:getRecentlyOpened" => ReadRecentlyOpened(),

		"workspaces:removeRecentlyOpened" => {
			let uri = arg_string(&arguments, 0);

			if !uri.is_empty() {
				MutateRecentlyOpened(|list| {
					if let Some(workspaces) = list.get_mut("workspaces").and_then(|v| v.as_array_mut()) {
						workspaces.retain(|e| e.get("uri").and_then(|v| v.as_str()).unwrap_or("") != uri);
					}

					if let Some(files) = list.get_mut("files").and_then(|v| v.as_array_mut()) {
						files.retain(|e| e.get("uri").and_then(|v| v.as_str()).unwrap_or("") != uri);
					}
				});
			}

			Ok(Value::Null)
		},

		"workspaces:addRecentlyOpened" => {
			let entries:Vec<Value> = arguments.first().and_then(|v| v.as_array()).cloned().unwrap_or_default();

			if !entries.is_empty() {
				MutateRecentlyOpened(|list| {
					let mut workspaces = list
						.get_mut("workspaces")
						.and_then(|v| v.as_array_mut())
						.cloned()
						.unwrap_or_default();

					let mut files = list
						.get_mut("files")
						.and_then(|v| v.as_array_mut())
						.cloned()
						.unwrap_or_default();

					for entry in entries {
						let folder = entry
							.get("folderUri")
							.cloned()
							.or_else(|| entry.get("workspace").and_then(|w| w.get("configPath").cloned()));

						let file = entry.get("fileUri").cloned();

						if let Some(folder_uri) = folder.and_then(|v| v.as_str().map(str::to_owned)) {
							workspaces.retain(|e| e.get("uri").and_then(|v| v.as_str()).unwrap_or("") != folder_uri);

							let mut item = serde_json::Map::new();

							item.insert("uri".into(), json!(&folder_uri));

							if let Some(label) = entry.get("label").and_then(|v| v.as_str().map(str::to_owned)) {
								item.insert("label".into(), json!(label));
							}

							workspaces.insert(0, Value::Object(item));
						}

						if let Some(file_uri) = file.and_then(|v| v.as_str().map(str::to_owned)) {
							files.retain(|e| e.get("uri").and_then(|v| v.as_str()).unwrap_or("") != file_uri);

							let mut item = serde_json::Map::new();

							item.insert("uri".into(), json!(file_uri));

							files.insert(0, Value::Object(item));
						}
					}

					workspaces.truncate(50);

					files.truncate(50);

					list.insert("workspaces".into(), Value::Array(workspaces));

					list.insert("files".into(), Value::Array(files));
				});
			}

			Ok(Value::Null)
		},

		"workspaces:clearRecentlyOpened" => {
			MutateRecentlyOpened(|list| {
				list.insert("workspaces".into(), json!([]));

				list.insert("files".into(), json!([]));
			});

			Ok(Value::Null)
		},

		"workspaces:enterWorkspace" | "workspaces:createUntitledWorkspace" | "workspaces:deleteUntitledWorkspace" => {
			Ok(Value::Null)
		},

		"workspaces:getWorkspaceIdentifier" => {
			let folders = runtime.Environment.ApplicationState.Workspace.GetWorkspaceFolders();

			if let Some(first) = folders.first() {
				use std::{
					collections::hash_map::DefaultHasher,
					hash::{Hash, Hasher},
				};

				let mut hasher = DefaultHasher::new();

				first.URI.as_str().hash(&mut hasher);

				let id = format!("{:016x}", hasher.finish());

				Ok(json!({ "id": id, "configPath": Value::Null, "uri": first.URI.to_string() }))
			} else {
				Ok(Value::Null)
			}
		},

		"workspaces:getDirtyWorkspaces" => Ok(json!([])),

		_ => Err(format!("Unknown workspace command: {}", command)),
	}
}
