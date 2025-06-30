// File: Mountain/Source/FileSystem/FileExplorerViewProvider.rs
// Role: A native (Rust-implemented) TreeViewProvider for the file explorer
// view. Responsibilities:
//   - Provide the root-level items (workspace folders).
//   - Provide the children for a given directory URI by reading the file
//     system.
//   - Construct `TreeItemDTO` JSON values for each file and directory.
//
// NOTE: This is a native provider, so it doesn't need to implement methods
// called *by* extensions (like `RegisterTreeDataProvider`). It only implements
// the "pull" methods called *by the host* (`GetChildren`, `GetTreeItem`).

//! # File Explorer View Provider
//!
//! A native (Rust-implemented) TreeViewProvider that provides the data for
//! the file explorer view.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::Arc;

use Common::{
	Effect::ApplicationRunTime::ApplicationRunTime,
	Environment::Environment::Environment,
	Error::CommonError::CommonError,
	FileSystem::{DTO::FileTypeDTO::FileTypeDTO, ReadDirectory::ReadDirectory},
	TreeView::TreeViewProvider::TreeViewProvider,
};
use async_trait::async_trait;
use log::info;
use serde_json::{Value, json};
// Import AppHandle and Manager trait
use tauri::{AppHandle, Manager};
use url::Url;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime;

#[derive(Clone)]
pub struct FileExplorerViewProvider {
	AppicationHandle:AppHandle,
}

impl Environment for FileExplorerViewProvider {}

impl FileExplorerViewProvider {
	pub fn New(AppicationHandle:AppHandle) -> Self { Self { AppicationHandle } }

	// Helper function to create the DTO, merged with V2's format
	fn CreateTreeItemDTO(&self, Name:&str, Uri:&Url, FileType:FileTypeDTO) -> Value {
		json!({

					"handle": Uri.to_string(),

					"label": { "label": Name },

		// 1: Collapsed, 0: None
					"collapsibleState": if FileType == FileTypeDTO::Directory { 1 } else { 0 },

					"resourceUri": json!({ "external": Uri.to_string() }),

					"command": if FileType == FileTypeDTO::File {

						Some(json!({

							"id": "vscode.open",

							"title": "Open File",

							"arguments": [json!({ "external": Uri.to_string() })]
						}))
					} else {

						None
					}

				})
	}
}

#[async_trait]
impl TreeViewProvider for FileExplorerViewProvider {
	// --- PUSH methods (not used by native providers) ---
	async fn RegisterTreeDataProvider(&self, _ViewIdentifier:String, _Options:Value) -> Result<(), CommonError> {
		Ok(())
	}

	async fn UnregisterTreeDataProvider(&self, _ViewIdentifier:String) -> Result<(), CommonError> { Ok(()) }

	async fn RevealTreeItem(
		&self,

		_ViewIdentifier:String,

		_ItemHandle:String,

		_Options:Value,
	) -> Result<(), CommonError> {
		Ok(())
	}

	async fn RefreshTreeView(&self, _ViewIdentifier:String, _ItemsToRefresh:Option<Value>) -> Result<(), CommonError> {
		Ok(())
	}

	async fn SetTreeViewMessage(&self, _ViewIdentifier:String, _Message:Option<String>) -> Result<(), CommonError> {
		Ok(())
	}

	async fn SetTreeViewTitle(
		&self,

		_ViewIdentifier:String,

		_Title:Option<String>,

		_Description:Option<String>,
	) -> Result<(), CommonError> {
		Ok(())
	}

	async fn SetTreeViewBadge(&self, _ViewIdentifier:String, _BadgeValue:Option<Value>) -> Result<(), CommonError> {
		Ok(())
	}

	// --- PULL methods (implemented by native providers) ---

	/// Retrieves the children for a given directory URI.
	async fn GetChildren(
		&self,

		// Kept for trait signature compatibility, but unused.
		_ViewIdentifier:String,

		ElementHandle:Option<String>,
	) -> Result<Vec<Value>, CommonError> {
		let RunTime = self.AppicationHandle.state::<Arc<MountainRunTime>>().inner().clone();

		let AppState = RunTime.Environment.ApplicationState.clone();

		let PathToRead = if let Some(Handle) = ElementHandle {
			// If an element is provided, it's a directory URI string.
			Url::parse(&Handle)
				.map_err(|_| {
					CommonError::InvalidArgument {
						ArgumentName:"ElementHandle".into(),

						Reason:"Handle is not a valid URI".into(),
					}
				})?
				.to_file_path()
				.map_err(|_| {
					CommonError::InvalidArgument {
						ArgumentName:"ElementHandle".into(),

						Reason:"Handle URI is not a file path".into(),
					}
				})?
		} else {
			// If no element, we are at the root. We should return the workspace folders.
			let Folders = AppState.WorkSpaceFolders.lock().unwrap();

			let RootItems:Vec<Value> = Folders
				.iter()
				.map(|folder| self.CreateTreeItemDTO(&folder.Name, &folder.URI, FileTypeDTO::Directory))
				.collect();

			return Ok(RootItems);
		};

		info!("[FileExplorer] Getting children for path: {}", PathToRead.display());

		// This now works because `RunTime` has the correct type and implements the
		// `ApplicationRunTime` trait.
		let Entries = RunTime.Run(ReadDirectory(PathToRead.clone())).await?;

		let Items = Entries
			.into_iter()
			.map(|(Name, FileType)| {
				let FullPath = PathToRead.join(&Name);

				let Uri = Url::from_file_path(FullPath).unwrap();

				self.CreateTreeItemDTO(&Name, &Uri, FileType)
			})
			.collect();

		Ok(Items)
	}

	/// Retrieves the `TreeItem` for a given handle (which is its URI).
	async fn GetTreeItem(&self, _ViewIdentifier:String, ElementHandle:String) -> Result<Value, CommonError> {
		let URI = Url::parse(&ElementHandle).map_err(|Error| {
			CommonError::InvalidArgument { ArgumentName:"ElementHandle".into(), Reason:Error.to_string() }
		})?;

		let Name = URI.path_segments().and_then(|s| s.last()).unwrap_or("").to_string();

		// Use robust check from V1
		let IsDirectory = URI.as_str().ends_with('/') || URI.to_file_path().map_or(false, |p| p.is_dir());

		let FileType = if IsDirectory { FileTypeDTO::Directory } else { FileTypeDTO::File };

		Ok(self.CreateTreeItemDTO(&Name, &URI, FileType))
	}
}
