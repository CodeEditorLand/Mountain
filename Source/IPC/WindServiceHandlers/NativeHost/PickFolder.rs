#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method: `nativeHost:pickFolderAndOpen`, `:pickFileAndOpen`,
//! `:pickFileFolderAndOpen`, `:pickWorkspaceAndOpen`.
//!
//! Atom I1 (2026-04-21): before webview reload, mutate
//! ApplicationState.Workspace and fire `$deltaWorkspaceFolders` to Cocoon so
//! extensions see the folder arrive synchronously.

use std::sync::Arc;

use serde_json::Value;
use tauri::{AppHandle, Manager};

use crate::{
	ApplicationState::{
		DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
		State::{
			ApplicationState::ApplicationState,
			WorkspaceState::WorkspaceDelta::UpdateWorkspaceFoldersAndBroadcast,
		},
	},
	dev_log,
};

pub async fn Fn(ApplicationHandle:AppHandle, _Arguments:Vec<Value>) -> Result<Value, String> {
	use std::path::PathBuf;

	use tauri_plugin_dialog::DialogExt;

	dev_log!("folder", "pickFolderAndOpen requested");

	let Handle = ApplicationHandle.clone();

	tokio::task::spawn_blocking(move || {
		let FolderPath = Handle.dialog().file().blocking_pick_folder();

		if let Some(Path) = FolderPath {
			let PathStr = Path.to_string();
			dev_log!("folder", "picked: {}", PathStr);

			if let Some(State) = Handle.try_state::<Arc<ApplicationState>>() {
				let PathBuf = PathBuf::from(&PathStr);
				let Canonical = PathBuf.canonicalize().unwrap_or(PathBuf.clone());
				if let Ok(Uri) = url::Url::from_directory_path(&Canonical) {
					let Name = Canonical
						.file_name()
						.and_then(|N| N.to_str())
						.map(str::to_string)
						.unwrap_or_else(|| Canonical.display().to_string());
					match WorkspaceFolderStateDTO::New(Uri, Name, 0) {
						Ok(Dto) => {
							dev_log!("folder", "pre-nav workspace-delta: broadcasting 1 folder to Cocoon");
							UpdateWorkspaceFoldersAndBroadcast(&Handle, &State.Workspace, vec![Dto]);
						},
						Err(Error) => {
							dev_log!(
								"folder",
								"warn: [pickFolderAndOpen] WorkspaceFolderStateDTO::New failed: {}",
								Error
							);
						},
					}
				} else {
					dev_log!(
						"folder",
						"warn: [pickFolderAndOpen] path → file URI conversion failed for {}",
						PathStr
					);
				}
			} else {
				dev_log!(
					"folder",
					"warn: [pickFolderAndOpen] ApplicationState not managed by Tauri - delta skipped"
				);
			}

			if let Some(Window) = Handle.get_webview_window("main") {
				if let Ok(CurrentUrl) = Window.url() {
					let Origin = CurrentUrl.origin().unicode_serialization();
					let EncodedPath = url::form_urlencoded::Serializer::new(String::new())
						.append_pair("folder", &PathStr)
						.finish();
					let NewUrl = format!("{}/?{}", Origin, EncodedPath);
					dev_log!("folder", "navigating: {}", NewUrl);
					let _ = Window.navigate(NewUrl.parse().unwrap());
					dev_log!("folder", "post-nav Window.navigate() returned; webview reloading");
				}
			}
		} else {
			dev_log!("folder", "pickFolderAndOpen cancelled by user");
		}
	});

	Ok(Value::Null)
}
