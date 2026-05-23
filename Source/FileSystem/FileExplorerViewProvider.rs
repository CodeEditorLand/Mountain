
//! Native TreeView provider for the workspace file explorer. Implements
//! `CommonLibrary::TreeView::TreeViewProvider`.
//!
//! Pull-only: `GetChildren` reads the workspace folders (when `ElementHandle`
//! is `None`) or the directory the handle points to. `GetTreeItem` builds a
//! single VS Code-shaped `TreeItemDTO`. Push methods are no-ops because the
//! provider is read-only and registered directly in `ApplicationState`.

use std::sync::Arc;

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Environment::Environment::Environment,
	Error::CommonError::CommonError,
	FileSystem::{DTO::FileTypeDTO::FileTypeDTO, ReadDirectory::ReadDirectory},
	TreeView::TreeViewProvider::TreeViewProvider,
};
use async_trait::async_trait;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager};
use url::Url;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime as Runtime, dev_log};

#[derive(Clone)]
pub struct Struct {
	AppicationHandle:AppHandle,
}

impl Environment for Struct {}

impl Struct {
	pub fn New(AppicationHandle:AppHandle) -> Self { Self { AppicationHandle } }

	fn CreateTreeItemDTO(&self, Name:&str, URI:&Url, FileType:FileTypeDTO) -> Value {
		json!({
			"handle": URI.to_string(),
			"label": { "label": Name },
			// 1 = collapsed, 0 = leaf.
			"collapsibleState": if FileType == FileTypeDTO::Directory { 1 } else { 0 },
			"resourceUri": json!({ "external": URI.to_string() }),
			"command": if FileType == FileTypeDTO::File {
				Some(json!({
					"id": "vscode.open",
					"title": "Open File",
					"arguments": [json!({ "external": URI.to_string() })]
				}))
			} else {
				None
			}
		})
	}
}

#[async_trait]
impl TreeViewProvider for Struct {
	// Push methods - no-ops for native providers.

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

	async fn OnTreeNodeExpanded(
		&self,

		_ViewIdentifier:String,

		_ElementHandle:String,

		_IsExpanded:bool,
	) -> Result<(), CommonError> {
		dev_log!("vfs", "[FileExplorer] OnTreeNodeExpanded - native provider no-op");

		Ok(())
	}

	async fn OnTreeSelectionChanged(
		&self,

		_ViewIdentifier:String,

		_SelectedHandles:Vec<String>,
	) -> Result<(), CommonError> {
		dev_log!("vfs", "[FileExplorer] OnTreeSelectionChanged - native provider no-op");

		Ok(())
	}

	async fn PersistTreeViewState(&self, _ViewIdentifier:String) -> Result<Value, CommonError> {
		Ok(json!({ "supported": false }))
	}

	async fn RestoreTreeViewState(&self, _ViewIdentifier:String, _StateValue:Value) -> Result<(), CommonError> {
		Ok(())
	}

	// Pull methods.

	async fn GetChildren(
		&self,

		_ViewIdentifier:String,

		ElementHandle:Option<String>,
	) -> Result<Vec<Value>, CommonError> {
		let RunTime = self.AppicationHandle.state::<Arc<Runtime>>().inner().clone();

		let AppState = RunTime.Environment.ApplicationState.clone();

		let PathToRead = if let Some(Handle) = ElementHandle {
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
			let Folders = AppState.Workspace.WorkspaceFolders.lock().unwrap();

			let RootItems:Vec<Value> = Folders
				.iter()
				.map(|Folder| self.CreateTreeItemDTO(&Folder.Name, &Folder.URI, FileTypeDTO::Directory))
				.collect();

			return Ok(RootItems);
		};

		dev_log!("vfs", "[FileExplorer] GetChildren {}", PathToRead.display());

		let Entries:Vec<(String, FileTypeDTO)> = RunTime.Run(ReadDirectory(PathToRead.clone())).await?;

		Ok(Entries
			.into_iter()
			.map(|(Name, FileType)| {
				let FullPath = PathToRead.join(&Name);
				let URI = Url::from_file_path(FullPath).unwrap();
				self.CreateTreeItemDTO(&Name, &URI, FileType)
			})
			.collect())
	}

	async fn GetTreeItem(&self, _ViewIdentifier:String, ElementHandle:String) -> Result<Value, CommonError> {
		let URI = Url::parse(&ElementHandle).map_err(|Error| {
			CommonError::InvalidArgument { ArgumentName:"ElementHandle".into(), Reason:Error.to_string() }
		})?;

		let Name = URI.path_segments().and_then(|S| S.last()).unwrap_or("").to_string();

		let IsDirectory = URI.as_str().ends_with('/') || URI.to_file_path().map_or(false, |P| P.is_dir());

		let FileType = if IsDirectory { FileTypeDTO::Directory } else { FileTypeDTO::File };

		Ok(self.CreateTreeItemDTO(&Name, &URI, FileType))
	}
}
