// @module WorkspaceProvider (Environment)
// @description Implements the `WorkspaceProvider` and `WorkspaceEditApplier`
// traits for the `MountainEnvironment` by delegating to the logic Handler.
// Renamed from `WorkSpaceProvider` for consistency.

use std::{path::PathBuf, sync::Arc};

use async_trait::async_trait;
use Common::{
	DTO::WorkspaceEditDto,
	Environment::Requires,
	error::CommonError,
	workspace::{WorkspaceEditApplier, WorkspaceProvider},
};
use log::info;
use serde_json::Value;
use url::Url;

use super::MountainEnvironment;
use crate::Handler::workspace as WorkspaceHandler;

#[async_trait]
impl WorkspaceProvider for MountainEnvironment {
	async fn GetWorkspaceFoldersInfo(&self) -> Result<Vec<(Url, String, usize)>, CommonError> {
		WorkspaceHandler::GetWorkspaceFoldersInfoLogic(&self.ApplicationHandle).await
	}

	async fn GetWorkspaceFolderInfo(&self, uri_to_match:Url) -> Result<Option<(Url, String, usize)>, CommonError> {
		WorkspaceHandler::GetWorkspaceFolderInfoLogic(&self.ApplicationHandle, uri_to_match).await
	}

	async fn GetWorkspaceName(&self) -> Result<Option<String>, CommonError> {
		WorkspaceHandler::GetWorkspaceNameLogic(&self.ApplicationHandle).await
	}

	async fn GetWorkspaceConfigurationPath(&self) -> Result<Option<PathBuf>, CommonError> {
		WorkspaceHandler::GetWorkspaceConfigurationPathLogic(&self.ApplicationHandle).await
	}

	async fn IsWorkspaceTrusted(&self) -> Result<bool, CommonError> {
		WorkspaceHandler::IsWorkspaceTrustedLogic(&self.ApplicationHandle).await
	}

	async fn RequestWorkspaceTrust(&self, options:Option<Value>) -> Result<bool, CommonError> {
		WorkspaceHandler::RequestWorkspaceTrustLogic(&self.ApplicationHandle, options).await
	}

	async fn FindFilesInWorkspace(
		&self,
		include_pattern_DTO:Value,
		exclude_pattern_DTO:Option<Value>,
		max_results:Option<usize>,
		use_ignore_files:bool,
		follow_symlinks:bool,
	) -> Result<Vec<Url>, CommonError> {
		WorkspaceHandler::FindFilesInWorkspaceLogic(
			&self.ApplicationHandle,
			include_pattern_DTO,
			exclude_pattern_DTO,
			max_results,
			use_ignore_files,
			follow_symlinks,
		)
		.await
	}

	async fn OpenFile(&self, path:PathBuf) -> Result<(), CommonError> {
		WorkspaceHandler::OpenFileLogic(&self.ApplicationHandle, path).await
	}
}

#[async_trait]
impl WorkspaceEditApplier for MountainEnvironment {
	async fn ApplyWorkspaceEdit(&self, edit_DTO:WorkspaceEditDto) -> Result<bool, CommonError> {
		info!(
			"[WorkspaceEditApplier] Applying WorkspaceEdit with {} edits.",
			edit_DTO.Edits.len()
		);
		// This would be delegated to a new handler in a full implementation.
		// For example: WorkspaceHandler::ApplyWorkspaceEditLogic(self, edit_DTO).await
		// The handler would then iterate through the edits and use the
		// DocumentProvider and FileSystemWriter effects to apply them.
		Ok(true)
	}
}

impl Requires<Arc<dyn WorkspaceProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WorkspaceProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn WorkspaceEditApplier + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn WorkspaceEditApplier + Send + Sync> { Arc::new(self.clone()) }
}
