//! Workspaces command router — all `workspaces:*` IPC commands.

use std::sync::Arc;

use serde_json::{Value, json};

use crate::{
	IPC::WindServiceHandlers::{
		Navigation::LabelGetWorkspace,
		UI::{WorkspacesAddFolder, WorkspacesGetFolders, WorkspacesGetName, WorkspacesRemoveFolder},
		Utilities::{
			JsonValueHelpers::{Fn as v_str, arg_string},
			RecentlyOpened::{Mutate::Fn as MutateRecentlyOpened, Read::Fn as ReadRecentlyOpened},
		},
		Workspaces::{CreateUntitledWorkspace, DeleteUntitledWorkspace, EnterWorkspace},
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes workspaces commands. Returns Some(result) for handled commands,
/// None otherwise.
pub(crate) async fn route(
	ApplicationHandle:tauri::AppHandle,

	RunTime:Arc<ApplicationRunTime>,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		// Primary workspace folder lifecycle
		"workspaces:getFolders" | "workspaces:getWorkspaceFolders" | "workspaces:getWorkspace" => {
			dev_log!("workspaces", "{}", command);

			Some(WorkspacesGetFolders::Fn(RunTime.clone()).await)
		},

		"workspaces:addFolder" | "workspaces:addWorkspaceFolders" => {
			dev_log!("workspaces", "{}", command);

			Some(WorkspacesAddFolder::Fn(RunTime.clone(), Arguments).await)
		},

		"workspaces:removeFolder" | "workspaces:removeWorkspaceFolders" => {
			dev_log!("workspaces", "{}", command);

			Some(WorkspacesRemoveFolder::Fn(RunTime.clone(), Arguments).await)
		},

		"workspaces:getName" => {
			dev_log!("workspaces", "{}", command);

			Some(WorkspacesGetName::Fn(RunTime.clone()).await)
		},

		// Recently-opened bookkeeping
		"workspaces:getRecentlyOpened" => {
			dev_log!("workspaces", "workspaces:getRecentlyOpened");

			Some(ReadRecentlyOpened())
		},

		"workspaces:removeRecentlyOpened" => {
			dev_log!("workspaces", "workspaces:removeRecentlyOpened");

			let Uri = arg_string(&Arguments, 0);

			if !Uri.is_empty() {
				MutateRecentlyOpened(|List| {
					if let Some(Workspaces) = List.get_mut("workspaces").and_then(|V| V.as_array_mut()) {
						Workspaces.retain(|Entry| Entry.get("uri").and_then(|V| V.as_str()).unwrap_or("") != Uri);
					}

					if let Some(Files) = List.get_mut("files").and_then(|V| V.as_array_mut()) {
						Files.retain(|Entry| Entry.get("uri").and_then(|V| V.as_str()).unwrap_or("") != Uri);
					}
				});
			}

			Some(Ok(Value::Null))
		},

		"workspaces:addRecentlyOpened" => {
			dev_log!("workspaces", "workspaces:addRecentlyOpened");

			// VS Code passes `[{ workspace?, folderUri?, fileUri?, label? }, …]`.
			let Entries:Vec<Value> = Arguments.first().and_then(|V| V.as_array()).cloned().unwrap_or_default();

			if !Entries.is_empty() {
				MutateRecentlyOpened(|List| {
					let Workspaces = List
						.get_mut("workspaces")
						.and_then(|V| V.as_array_mut())
						.map(|V| std::mem::take(V))
						.unwrap_or_default();

					let Files = List
						.get_mut("files")
						.and_then(|V| V.as_array_mut())
						.map(|V| std::mem::take(V))
						.unwrap_or_default();

					let mut MergedWorkspaces = Workspaces;

					let mut MergedFiles = Files;

					for Entry in Entries {
						let Folder = Entry
							.get("folderUri")
							.cloned()
							.or_else(|| Entry.get("workspace").and_then(|W| W.get("configPath").cloned()));

						let File = Entry.get("fileUri").cloned();

						if let Some(FolderUri) = Folder.and_then(|V| v_str(&V)) {
							MergedWorkspaces
								.retain(|E| E.get("uri").and_then(|V| V.as_str()).unwrap_or("") != FolderUri);

							let mut Item = serde_json::Map::new();

							Item.insert("uri".into(), json!(FolderUri));

							if let Some(Label) = Entry.get("label").and_then(|V| V.as_str()) {
								Item.insert("label".into(), json!(Label));
							}

							MergedWorkspaces.insert(0, Value::Object(Item));
						}

						if let Some(FileUri) = File.and_then(|V| v_str(&V)) {
							MergedFiles.retain(|E| E.get("uri").and_then(|V| V.as_str()).unwrap_or("") != FileUri);

							let mut Item = serde_json::Map::new();

							Item.insert("uri".into(), json!(FileUri));

							MergedFiles.insert(0, Value::Object(Item));
						}
					}

					// Cap at 50 each - matches VS Code's default.
					MergedWorkspaces.truncate(50);

					MergedFiles.truncate(50);

					List.insert("workspaces".into(), Value::Array(MergedWorkspaces));

					List.insert("files".into(), Value::Array(MergedFiles));
				});
			}

			Some(Ok(Value::Null))
		},

		"workspaces:clearRecentlyOpened" => {
			dev_log!("workspaces", "workspaces:clearRecentlyOpened");

			MutateRecentlyOpened(|List| {
				List.insert("workspaces".into(), json!([]));

				List.insert("files".into(), json!([]));
			});

			Some(Ok(Value::Null))
		},

		// Workspace enter/create/delete
		"workspaces:enterWorkspace" => {
			Some(EnterWorkspace::Fn(ApplicationHandle.clone(), RunTime.clone(), Arguments).await)
		},

		"workspaces:createUntitledWorkspace" => Some(CreateUntitledWorkspace::Fn(ApplicationHandle.clone()).await),

		"workspaces:deleteUntitledWorkspace" => {
			Some(DeleteUntitledWorkspace::Fn(ApplicationHandle.clone(), Arguments).await)
		},

		// Workspace identifier derivation (hash of first folder URI)
		"workspaces:getWorkspaceIdentifier" => {
			let Workspace = &RunTime.Environment.ApplicationState.Workspace;

			let Folders = Workspace.GetWorkspaceFolders();

			if let Some(First) = Folders.first() {
				use std::{
					collections::hash_map::DefaultHasher,
					hash::{Hash, Hasher},
				};

				let mut Hasher = DefaultHasher::new();

				First.URI.as_str().hash(&mut Hasher);

				let Id = format!("{:016x}", Hasher.finish());

				Some(Ok(json!({
					"id": Id,
					"configPath": Value::Null,
					"uri": First.URI.to_string(),
				})))
			} else {
				Some(Ok(Value::Null))
			}
		},

		// Workspace display name (delegates to LabelGetWorkspace)
		"workspaces:getWorkspaceName" => Some(LabelGetWorkspace::Fn(RunTime.clone()).await),

		// Hot-exit backup check — Mountain has no backup service.
		"workspaces:getDirtyWorkspaces" => Some(Ok(json!([]))),

		_ => None,
	}
}
