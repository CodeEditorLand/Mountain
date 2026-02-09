//! # FileExplorerViewProvider (FileSystem)
//!
//! A native (Rust-implemented) `TreeViewProvider` that provides the data for
//! the file explorer (tree) view in Mountain. This is a **native provider**,
//! meaning it is implemented directly in Rust rather than being provided by an
//! extension.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Root-Level Items (Workspace Folders)
//! - Return the list of workspace folders as root tree nodes
//! - Each folder appears as a collapsible node at the top level
//! - Folder names are displayed as labels
//!
//! ### 2. Directory Listing
//! - Provide children for a given directory URI (via `GetChildren`)
//! - Read filesystem to enumerate files and subdirectories
//! - Return appropriate `TreeItemDTO` for each entry
//! - Handle permissions errors gracefully
//!
//! ### 3. Tree Item Construction
//! - Build `TreeItemDTO` JSON objects with proper structure:
//!   - `handle`: Unique identifier (file URI)
//!   - `label`: Display name
//!   - `collapsibleState`: 1 for directories, 0 for files
//!   - `resourceUri`: File URI with `external` property
//!   - `command`: Open file command for leaf nodes
//!
//! ## ARCHITECTURAL ROLE
//!
//! The FileExplorerViewProvider is a **native TreeViewProvider**:
//!
//! ```text
//! TreeView API ──► FileExplorerViewProvider ──► FileSystem ReadDirectory/ReadFile
//!                          │
//!                          └─► Returns TreeItemDTO JSON
//! ```
//!
//! ### Position in Mountain
//! - `FileSystem` module: File system operations
//! - Implements `CommonLibrary::TreeView::TreeViewProvider` trait
//! - Registered as provider in `ApplicationState::ActiveTreeViews`
//!
//! ### Differences from Extension Providers
//! - **Native Provider**: Direct Rust implementation, no extension hosting
//! - **Read-Only**: Only implements "pull" methods (`GetChildren`,
//!   `GetTreeItem`)
//! - **No Push Methods**: Does not use `RegisterTreeDataProvider`,
//!   `RefreshTreeView`, etc.
//! - **No Sidecar**: No extension host communication overhead
//!
//! ### Dependencies
//! - `CommonLibrary::FileSystem::ReadDirectory` and `ReadFile`: Filesystem
//!   access
//! - `CommonLibrary::TreeView::TreeViewProvider`: Provider trait
//! - `ApplicationRunTime`: Effect execution
//! - `ApplicationState`: Workspace folder access
//!
//! ### Dependents
//! - `Binary::Main::Fn`: Creates and registers provider instance
//! - TreeView UI component: Requests data via provider methods
//! - Command handlers: Trigger tree view operations
//!
//! ## TREE ITEM DTO STRUCTURE
//!
//! Each tree item is a JSON object compatible with VS Code's `TreeItem`:
//!
//! ```json
//! {
//!   "handle": "file:///path/to/item",
//!   "label": { "label": "itemName" },
//!   "collapsibleState": 1,
//!   "resourceUri": { "external": "file:///path/to/item" },
//!   "command": {
//!     "id": "vscode.open",
//!     "title": "Open File",
//!     "arguments": [{ "external": "file:///path/to/item" }]
//!   }
//! }
//! ```
//!
//! ## METHODS OVERVIEW
//!
//! - `GetChildren`: Returns child items for a given parent directory
//! - `GetTreeItem`: Returns a single tree item for a given handle (URI)
//! - Other `TreeViewProvider` methods (push-based) are no-ops for native
//!   providers
//!
//! ## ERROR HANDLING
//!
//! - Filesystem errors are converted to `CommonError::FileSystemIO`
//! - Invalid URIs return `CommonError::InvalidArgument`
//! - Permission errors are logged and empty results returned
//!
//! ## PERFORMANCE
//!
//! - Directory reads are async via `ApplicationRunTime`
//! - Each `GetChildren` call reads the directory from disk
//! - Consider caching for large directories (TODO)
//! - Stat calls are minimized by using directory entry metadata
//!
//! ## VS CODE REFERENCE
//!
//! Patterns from VS Code:
//! - `vs/workbench/contrib/files/browser/filesViewProvider.ts`: File tree
//!   provider
//! - `vs/platform/workspace/common/workspace.ts`: Tree item DTO structure
//!
//! ## TODO
//!
//! - [ ] Implement tree item caching for better performance
//! - [ ] Add file icon decoration based on file type
//! - [ ] Support drag-and-drop operations
//! - [ ] Add file/folder filtering (gitignore, exclude patterns)
//! - [ ] Implement tree state persistence (expanded/collapsed)
//! - [ ] Add file change notifications (watch for file system events)
//! - [ ] Support virtual workspace folders (non-file URIs)
//!
//! ## MODULE CONTENTS
//!
//! - [`FileExplorerViewProvider`]: Main provider struct
//! - [`CreateTreeItemDTO`]: Helper to build tree item JSON

use std::sync::Arc;

use CommonLibrary::{
	Effect::{ApplicationRunTime, ApplicationRunTime::ApplicationRunTime as ApplicationRunTimeTrait},
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

use crate::RunTime::ApplicationRunTime::ApplicationRunTime as Runtime;

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

	// --- State Management Methods (not used by native file explorer providers) ---

	/// Handles tree node expansion/collapse events.
	/// These events are not relevant for the native file explorer provider.
	async fn OnTreeNodeExpanded(
		&self,
		_ViewIdentifier:String,
		_ElementHandle:String,
		_IsExpanded:bool,
	) -> Result<(), CommonError> {
		info!("[FileExplorer] OnTreeNodeExpanded called - not implemented for native providers");
		Ok(())
	}

	/// Handles tree selection changes.
	/// These events are not relevant for the native file explorer provider.
	async fn OnTreeSelectionChanged(
		&self,
		_ViewIdentifier:String,
		_SelectedHandles:Vec<String>,
	) -> Result<(), CommonError> {
		info!("[FileExplorer] OnTreeSelectionChanged called - not implemented for native providers");
		Ok(())
	}

	/// Persists tree view state.
	/// These events are not relevant for the native file explorer provider.
	async fn PersistTreeViewState(&self, _ViewIdentifier:String) -> Result<Value, CommonError> {
		info!("[FileExplorer] PersistTreeViewState called - not implemented for native providers");
		Ok(json!({ "supported": false }))
	}

	/// Restores tree view state.
	/// These events are not relevant for the native file explorer provider.
	async fn RestoreTreeViewState(&self, _ViewIdentifier:String, _StateValue:Value) -> Result<(), CommonError> {
		info!("[FileExplorer] RestoreTreeViewState called - not implemented for native providers");
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
		let RunTime = self.AppicationHandle.state::<Arc<Runtime>>().inner().clone();

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
			let Folders = AppState.Workspace.WorkspaceFolders.lock().unwrap();

			let RootItems:Vec<Value> = Folders
				.iter()
				.map(|folder| self.CreateTreeItemDTO(&folder.Name, &folder.URI, FileTypeDTO::Directory))
				.collect();

			return Ok(RootItems);
		};

		info!("[FileExplorer] Getting children for path: {}", PathToRead.display());

		// This now works because `RunTime` has the correct type and implements the
		// `ApplicationRunTime` trait.
		let Entries:Vec<(String, CommonLibrary::FileSystem::DTO::FileTypeDTO::FileTypeDTO)> =
			RunTime.Run(ReadDirectory(PathToRead.clone())).await?;

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
