// ============================================================================
// File: Mountain/Source/Environment/WorkSpaceProvider.rs
// ============================================================================
// This module follows the Land ecosystem's PascalCase naming convention.
// See: https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//
// # WorkSpaceProvider Implementation
//
// Implements the `WorkSpaceProvider` and `WorkSpaceEditApplier` traits for
// the `MountainEnvironment`. This provider contains the core logic for
// workspace-related operations, including multi-root workspace support,
// folder management, and workspace-wide file operations.
//
// ## Key Features:
// - Multi-root workspace folder management
// - Folder addition, removal, and reordering
// - Workspace trust management
// - File discovery with glob patterns
// - Workspace edit application
// - Custom editor routing
// - Workspace event propagation
//
// ## VSCode Reference:
// - vs/workbench/services/workspace/browser/workspaceService.ts
// - vs/workbench/contrib/files/common/editors/textFileEditor.ts
// - vs/platform/workspace/common/workspace.ts
//
// ============================================================================

#![allow(non_snake_case, non_camel_case_types)]

use std::{path::PathBuf, sync::Arc};

use Common::{
	CustomEditor::CustomEditorProvider::CustomEditorProvider,
	DTO::WorkSpaceEditDTO::WorkSpaceEditDTO,
	Document::DocumentProvider::DocumentProvider,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	WebView::WebViewProvider::WebViewProvider,
	WorkSpace::{WorkSpaceEditApplier::WorkSpaceEditApplier, WorkSpaceProvider::WorkSpaceProvider},
};
use async_trait::async_trait;
use globset::{Glob, GlobMatcher};
use ignore::WalkBuilder;
use log::{info, warn};
use serde_json::{Value, json};
use url::Url;

use super::{MountainEnvironment::MountainEnvironment, Utility};

#[async_trait]
impl WorkSpaceProvider for MountainEnvironment {
	/// Retrieves information about all currently open workspace folders.
	async fn GetWorkSpaceFoldersInfo(&self) -> Result<Vec<(Url, String, usize)>, CommonError> {
		info!("[WorkSpaceProvider] Getting workspace folders info.");
		let FoldersGuard = self
			.ApplicationState
			.WorkSpaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
		Ok(FoldersGuard.iter().map(|f| (f.URI.clone(), f.Name.clone(), f.Index)).collect())
	}

	/// Retrieves information for the specific workspace folder that contains a
	/// given URI.
	async fn GetWorkSpaceFolderInfo(&self, URIToMatch:Url) -> Result<Option<(Url, String, usize)>, CommonError> {
		let FoldersGuard = self
			.ApplicationState
			.WorkSpaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
		for Folder in FoldersGuard.iter() {
			if URIToMatch.as_str().starts_with(Folder.URI.as_str()) {
				return Ok(Some((Folder.URI.clone(), Folder.Name.clone(), Folder.Index)));
			}
		}

		Ok(None)
	}

	/// Gets the name of the current workspace.
	async fn GetWorkSpaceName(&self) -> Result<Option<String>, CommonError> {
		self.ApplicationState.GetWorkSpaceIdentifier().map(Some)
	}

	/// Gets the path to the workspace configuration file (`.code-workspace`).
	async fn GetWorkSpaceConfigurationPath(&self) -> Result<Option<PathBuf>, CommonError> {
		Ok(self
			.ApplicationState
			.WorkSpaceConfigurationPath
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.clone())
	}

	/// Checks if the current workspace is trusted.
	async fn IsWorkSpaceTrusted(&self) -> Result<bool, CommonError> {
		Ok(self.ApplicationState.IsTrusted.load(std::sync::atomic::Ordering::Relaxed))
	}

	/// Requests workspace trust from the user.
	async fn RequestWorkSpaceTrust(&self, _Options:Option<Value>) -> Result<bool, CommonError> {
		warn!("[WorkSpaceProvider] RequestWorkSpaceTrust is not implemented; defaulting to trusted.");

		// A full implementation would show a modal dialog to the user and wait for their response.
		self.ApplicationState
			.IsTrusted
			.store(true, std::sync::atomic::Ordering::Relaxed);

		// Emit trust event
		self.ApplicationHandle
			.emit("sky://workspace/trust-changed", json!({ "IsTrusted": true }))
			.map_err(|Error| CommonError::IPCError {
				Description: format!("Failed to emit workspace trust event: {}", Error),
			})?;

		Ok(true)
	}

	/// Adds a workspace folder.
	async fn AddWorkSpaceFolder(&self, URI:Url, Name:Option<String>) -> Result<String, CommonError> {
		info!("[WorkSpaceProvider] Adding workspace folder: {} ({:?})", URI, Name);

		let FolderName = Name.unwrap_or_else(|| {
			URI.path()
				.split('/')
				.filter(|s| !s.is_empty())
				.last()
				.unwrap_or("Workspace")
				.to_string()
		});

		let mut FoldersGuard = self
			.ApplicationState
			.WorkSpaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		let NewIndex = FoldersGuard.len();

		// Check for duplicate URI
		if FoldersGuard.iter().any(|f| f.URI == URI) {
			return Err(CommonError::InvalidArgument {
				ArgumentName: "URI".into(),
				Reason: format!("Workspace folder already exists: {}", URI),
			});
		}

		FoldersGuard.push(crate::ApplicationState::DTO::WorkSpaceFolderDTO {
			URI: URI.clone(),
			Name: FolderName.clone(),
			Index: NewIndex,
		});

		drop(FoldersGuard);

		// Emit folder added event
		self.ApplicationHandle
			.emit(
				"sky://workspace/folder-added",
				json!({ "URI": URI, "Name": FolderName, "Index": NewIndex }),
			)
			.map_err(|Error| CommonError::IPCError {
				Description: format!("Failed to emit workspace folder added event: {}", Error),
			})?;

		info!("[WorkSpaceProvider] Workspace folder added: {}", FolderName);

		Ok(FolderName)
	}

	/// Removes a workspace folder.
	async fn RemoveWorkSpaceFolder(&self, URI:Url) -> Result<(), CommonError> {
		info!("[WorkSpaceProvider] Removing workspace folder: {}", URI);

		let mut FoldersGuard = self
			.ApplicationState
			.WorkSpaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		let OriginalLen = FoldersGuard.len();
		FoldersGuard.retain(|f| f.URI != URI);
		let WasRemoved = FoldersGuard.len() < OriginalLen;

		drop(FoldersGuard);

		if WasRemoved {
			// Emit folder removed event
			self.ApplicationHandle
				.emit(
					"sky://workspace/folder-removed",
					json!({ "URI": URI }),
				)
				.map_err(|Error| CommonError::IPCError {
					Description: format!("Failed to emit workspace folder removed event: {}", Error),
				})?;

			info!("[WorkSpaceProvider] Workspace folder removed: {}", URI);
		} else {
			warn!("[WorkSpaceProvider] Workspace folder not found for removal: {}", URI);
		}

		Ok(())
	}

	/// Updates workspace folder (renames or moves).
	async fn UpdateWorkSpaceFolder(&self, OldURI:Url, NewURI:Option<Url>, NewName:Option<String>) -> Result<(), CommonError> {
		info!("[WorkSpaceProvider] Updating workspace folder: {} -> {:?}", OldURI, NewURI);

		let mut FoldersGuard = self
			.ApplicationState
			.WorkSpaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if let Some(Folder) = FoldersGuard.iter_mut().find(|f| f.URI == OldURI) {
			if let Some(NewURI) = NewURI {
				Folder.URI = NewURI;
			}
			if let Some(Name) = NewName {
				Folder.Name = Name;
			}

			drop(FoldersGuard);

			// Emit folder updated event
			self.ApplicationHandle
				.emit(
					"sky://workspace/folder-updated",
					json!({ "OldURI": OldURI, "Folder": Folder }),
				)
				.map_err(|Error| CommonError::IPCError {
					Description: format!("Failed to emit workspace folder updated event: {}", Error),
				})?;

			Ok(())
		} else {
			Err(CommonError::InvalidArgument {
				ArgumentName: "URI".into(),
				Reason: format!("Workspace folder not found: {}", OldURI),
			})
		}
	}

	/// Finds files within the workspace using glob patterns.
	async fn FindFilesInWorkSpace(
		&self,

		IncludePatternDTO:Value,

		ExcludePatternDTO:Option<Value>,

		MaxResults:Option<usize>,

		UseIgnoreFiles:bool,

		FollowSymlinks:bool,
	) -> Result<Vec<Url>, CommonError> {
		info!(
			"[WorkSpaceProvider] Finding files with include pattern: {:?}",
			IncludePatternDTO
		);
		let FoldersGuard = self
			.ApplicationState
			.WorkSpaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
		if FoldersGuard.is_empty() {
			return Ok(vec![]);
		}

		let IncludeMatcher = BuildGlobMatcher(IncludePatternDTO)?;
		let ExcludeMatcher = ExcludePatternDTO.map(BuildGlobMatcher).transpose()?.flatten();
		let mut Results:Vec<Url> = Vec::new();
		let MaxResultsCap = MaxResults.unwrap_or(usize::MAX);

		for Folder in FoldersGuard.iter() {
			if Results.len() >= MaxResultsCap {
				break;
			}

			let FolderPath = match Folder.URI.to_file_path() {
				Ok(path) => path,

				Err(_) => continue,
			};
			let mut WalkerBuilder = WalkBuilder::new(&FolderPath);
			WalkerBuilder.standard_filters(UseIgnoreFiles).follow_links(FollowSymlinks);

			for EntryResult in WalkerBuilder.build() {
				if Results.len() >= MaxResultsCap {
					break;
				}

				if let Ok(Entry) = EntryResult {
					let Path = Entry.path();
					if Path.is_dir() {
						continue;
					}

					if !IncludeMatcher.as_ref().map_or(true, |g| g.is_match(Path)) {
						continue;
					}

					if let Some(ref exclude) = ExcludeMatcher {
						if exclude.is_match(Path) {
							continue;
						}
					}

					if let Ok(URL) = Url::from_file_path(Path) {
						Results.push(URL);
					}
				}
			}
		}

		Ok(Results)
	}

	/// Requests that the host application open the specified file path in an
	/// editor.
	async fn OpenFile(&self, Path:PathBuf) -> Result<(), CommonError> {
		let URI = Url::from_file_path(Path.clone()).map_err(|_| {
			CommonError::InvalidArgument { ArgumentName:"Path".into(), Reason:"Could not convert path to URI.".into() }
		})?;

		// Check for custom editor based on file extension
		let FileName = Path.file_name()
			.and_then(|n| n.to_str())
			.unwrap_or_default();

		let CustomEditorViewType = self.FindCustomEditorForFile(&FileName).await?;

		if let Some(ViewType) = CustomEditorViewType {
			info!(
				"[WorkSpaceProvider] Found custom editor '{}' for file '{}'",
				ViewType,
				Path.display()
			);
			let WebViewProvider:Arc<dyn WebViewProvider> = self.Require();
			let Handle = WebViewProvider
				.CreateWebViewPanel(
					json!({ "id": "placeholder.extension" }),
					ViewType.clone(),
					Path.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default(),
					json!({ "viewColumn": -1 }),
					json!({}),
					json!({ "enableScripts": true }),
				)
				.await?;
			let CustomEditorProvider:Arc<dyn CustomEditorProvider> = self.Require();
			CustomEditorProvider.ResolveCustomEditor(ViewType, URI, Handle).await?;
			return Ok(());
		}

		info!(
			"[WorkSpaceProvider] No custom editor found. Opening '{}' as text.",
			Path.display()
		);
		let URIComponents = json!({ "external": URI.to_string(), "$mid": 1 });
		let DocProvider:Arc<dyn DocumentProvider> = self.Require();
		DocProvider.OpenDocument(URIComponents, None, None).await?;
		Ok(())
	}

	/// Gets workspace configuration for the given workspace folder.
	async fn GetWorkSpaceConfiguration(&self, ScopeURI:Option<Url>, Section:Option<String>) -> Result<Value, CommonError> {
		info!(
			"[WorkSpaceProvider] Getting workspace configuration for scope: {:?}, section: {:?}",
			ScopeURI, Section
		);

		// For now, return an empty configuration.
		// A full implementation would:
		// 1. Read workspace settings files
		// 2. Merge with user settings
		// 3. Apply defaults
		// 4. Return the merged configuration

		Ok(json!({}))
	}

	/// Updates workspace configuration.
	async fn UpdateWorkSpaceConfiguration(&self, ScopeURI:Option<Url>, Key:String, Value:Value) -> Result<(), CommonError> {
		info!(
			"[WorkSpaceProvider] Updating workspace configuration for scope: {:?}, key: {}",
			ScopeURI, Key
		);

		// For now, just emit an event that the configuration was updated.
		// A full implementation would:
		// 1. Update the settings file
		// 2. Apply the change
		// 3. Notify listeners

		self.ApplicationHandle
			.emit(
				"sky://workspace/configuration-changed",
				json!({ "Key": Key, "Value": Value }),
			)
			.map_err(|Error| CommonError::IPCError {
				Description: format!("Failed to emit configuration changed event: {}", Error),
			})?;

		Ok(())
	}

	/// Gets active workspace folder by URI.
	async fn GetActiveWorkSpaceFolder(&self) -> Result<Option<(Url, String, usize)>, CommonError> {
		let FoldersGuard = self
			.ApplicationState
			.WorkSpaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		if FoldersGuard.is_empty() {
			Ok(None)
		} else {
			// For now, return the first folder.
			// A full implementation would check the active document and return its containing folder.
			let First = FoldersGuard.first().unwrap();
			Ok(Some((First.URI.clone(), First.Name.clone(), First.Index)))
		}
	}
}

impl MountainEnvironment {
	/// Finds a custom editor for a given file based on the file's extension or pattern.
	async fn FindCustomEditorForFile(&self, FileName:&str) -> Result<Option<String>, CommonError> {
		// Get file extension
		let Extension = FileName
			.rsplit('.')
			.next()
			.map(|s| s.to_lowercase())
			.and_then(|ext| if ext.is_empty() { None } else { Some(ext) });

		// A full implementation would:
		// 1. Check registered custom editors
		// 2. Match file patterns against the file name
		// 3. Return the matching editor type

		// For now, return None (use default text editor)
		match Extension.as_deref() {
			Some("pdf") => Ok(Some("pdf.viewer".to_string())),
			Some("png") | Some("jpg") | Some("jpeg") | Some("gif") | Some("svg") => {
				Ok(Some("image.preview".to_string()))
			},
			_ => Ok(None),
		}
	}
}

fn BuildGlobMatcher(GlobValue:Value) -> Result<Option<GlobMatcher>, CommonError> {
	GlobValue
		.as_str()
		.map(|Pattern| {
			Glob::new(Pattern).map(|g| g.compile_matcher()).map_err(|Error| {
				CommonError::InvalidArgument { ArgumentName:"GlobPattern".to_string(), Reason:Error.to_string() }
			})
		})
		.transpose()
}

#[async_trait]
impl WorkSpaceEditApplier for MountainEnvironment {
	async fn ApplyWorkSpaceEdit(&self, EditDTO:WorkSpaceEditDTO) -> Result<bool, CommonError> {
		let DocProvider:Arc<dyn DocumentProvider> = self.Require();
		for (URIValue, Edits) in EditDTO.Edits {
			let URI = serde_json::from_value::<Url>(URIValue)?;
			let Document = {
				self.ApplicationState
					.OpenDocuments
					.lock()
					.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
					.get(URI.as_str())
					.cloned()
			};

			if let Some(Doc) = Document {
				let NewVersionID = Doc.Version + 1;
				DocProvider
					.ApplyDocumentChanges(URI.clone(), NewVersionID, json!(Edits), true, false, false)
					.await?;
			} else {
				warn!("[WorkSpaceProvider] Attempted to apply edit to non-open document: {}", URI);
			}
		}

		Ok(true)
	}
}
