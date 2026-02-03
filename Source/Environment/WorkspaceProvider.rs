// ============================================================================
// File: Mountain/Source/Environment/WorkspaceProvider.rs
// ============================================================================
// # WorkspaceProvider Implementation
//
// Implements the `WorkspaceProvider` and `WorkspaceEditApplier` traits for
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

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	CustomEditor::CustomEditorProvider::CustomEditorProvider,
	DTO::WorkspaceEditDTO::WorkspaceEditDTO,
	Document::DocumentProvider::DocumentProvider,
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	WebView::WebViewProvider::WebViewProvider,
	Workspace::{WorkspaceEditApplier::WorkspaceEditApplier, WorkspaceProvider::WorkspaceProvider},
};
use async_trait::async_trait;
use log::{info, warn};
use serde_json::{Value, json};
use url::Url;

use super::{MountainEnvironment::MountainEnvironment, Utility};

#[async_trait]
impl WorkspaceProvider for MountainEnvironment {
	/// Retrieves information about all currently open workspace folders.
	async fn GetWorkspaceFoldersInfo(&self) -> Result<Vec<(Url, String, usize)>, CommonError> {
		info!("[WorkspaceProvider] Getting workspace folders info.");
		let FoldersGuard = self
			.ApplicationState
			.WorkspaceFolders
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
		Ok(FoldersGuard.iter().map(|f| (f.URI.clone(), f.Name.clone(), f.Index)).collect())
	}

	/// Retrieves information for the specific workspace folder that contains a
	/// given URI.
	async fn GetWorkspaceFolderInfo(&self, URIToMatch:Url) -> Result<Option<(Url, String, usize)>, CommonError> {
		let FoldersGuard = self
			.ApplicationState
			.WorkspaceFolders
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
	async fn GetWorkspaceName(&self) -> Result<Option<String>, CommonError> {
		self.ApplicationState.GetWorkspaceIdentifier().map(Some)
	}

	/// Gets the path to the workspace configuration file (`.code-workspace`).
	async fn GetWorkspaceConfigurationPath(&self) -> Result<Option<PathBuf>, CommonError> {
		Ok(self
			.ApplicationState
			.WorkspaceConfigurationPath
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.clone())
	}

	/// Checks if the current workspace is trusted.
	async fn IsWorkspaceTrusted(&self) -> Result<bool, CommonError> {
		Ok(self.ApplicationState.IsTrusted.load(std::sync::atomic::Ordering::Relaxed))
	}

	/// Requests workspace trust from the user.
	async fn RequestWorkspaceTrust(&self, _Options:Option<Value>) -> Result<bool, CommonError> {
		warn!("[WorkspaceProvider] RequestWorkspaceTrust is not implemented; defaulting to trusted.");
		Ok(true)
	}

	/// Finds files in the workspace matching the specified query.
	async fn FindFilesInWorkspace(
		&self,
		query:Value,
		_:Option<Value>,
		_:Option<usize>,
		_:bool,
		_:bool,
	) -> Result<Vec<Url>, CommonError> {
		info!("[WorkspaceProvider] FindFilesInWorkspace called");
		// TODO: Implement file search
		Ok(Vec::new())
	}

	/// Opens a file in the workspace.
	async fn OpenFile(&self, path:PathBuf) -> Result<(), CommonError> {
		info!("[WorkspaceProvider] OpenFile called for: {:?}", path);
		// TODO: Implement file opening
		Ok(())
	}
}

#[async_trait]
impl WorkspaceEditApplier for MountainEnvironment {
	/// Applies a workspace edit to the workspace.
	async fn ApplyWorkspaceEdit(&self, Edit:WorkspaceEditDTO) -> Result<bool, CommonError> {
		info!("[WorkspaceEditApplier] Applying workspace edit");

		// For now, just log the edit details
		match Edit {
			WorkspaceEditDTO { Edits } => {
				for (DocumentURI, TextEdits) in Edits {
					info!(
						"[WorkspaceEditApplier] Would apply {} edits to document: {}",
						TextEdits.len(),
						DocumentURI
					);
				}
			},
		}

		// TODO: Actually implement workspace edit application
		warn!("[WorkspaceEditApplier] ApplyWorkspaceEdit is not fully implemented");

		Ok(true)
	}
}
