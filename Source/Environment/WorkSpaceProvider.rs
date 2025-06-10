use std::{path::PathBuf, sync::Arc};

use Common::{
	dto::WorkspaceEditDto,
	environment::Requires,
	error::CommonError,
	workspace::{WorkspaceEditApplier, WorkspaceProvider},
};
use async_trait::async_trait;
use log::info;
use serde_json::Value;
use url::Url;

// @module WorkspaceProvider (Environment)
// @description Implements the `WorkspaceProvider` and `WorkspaceEditApplier`
// traits for the `MountainEnvironment` by delegating to the logic Handler.
use crate::{Handler::workspace as WorkspaceHandler, environment::MountainEnvironment};

#[async_trait]
impl WorkspaceProvider for MountainEnvironment {
	async fn GetWorkspaceFoldersInfo(&self) -> Result<Vec<(Url, String, usize)>, CommonError> {
		WorkspaceHandler::GetWorkspaceFoldersInfoLogic(&self.ApplicationHandle).await
	}

	async fn GetWorkspaceFolderInfo(&self, UriToMatch:Url) -> Result<Option<(Url, String, usize)>, CommonError> {
		WorkspaceHandler::GetWorkspaceFolderInfoLogic(&self.ApplicationHandle, UriToMatch).await
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

	async fn RequestWorkspaceTrust(&self, Options:Option<Value>) -> Result<bool, CommonError> {
		WorkspaceHandler::RequestWorkspaceTrustLogic(&self.ApplicationHandle, Options).await
	}

	async fn FindFilesInWorkspace(
		&self,
		IncludePatternDto:Value,
		ExcludePatternDto:Option<Value>,
		MaxResults:Option<usize>,
		UseIgnoreFiles:bool,
		FollowSymlinks:bool,
	) -> Result<Vec<Url>, CommonError> {
		WorkspaceHandler::FindFilesInWorkspaceLogic(
			&self.ApplicationHandle,
			IncludePatternDto,
			ExcludePatternDto,
			MaxResults,
			UseIgnoreFiles,
			FollowSymlinks,
		)
		.await
	}

	async fn OpenFile(&self, Path:PathBuf) -> Result<(), CommonError> {
		WorkspaceHandler::OpenFileLogic(&self.ApplicationHandle, Path).await
	}
}

#[async_trait]
impl WorkspaceEditApplier for MountainEnvironment {
	async fn ApplyWorkspaceEdit(&self, EditDto:WorkspaceEditDto) -> Result<bool, CommonError> {
		info!(
			"[WorkspaceEditApplier] Applying WorkspaceEdit with {} edits.",
			EditDto.Edits.len()
		);
		// This would be delegated to a new handler in a full implementation.
		// For example: WorkspaceHandler::ApplyWorkspaceEditLogic(self, EditDto).await
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
