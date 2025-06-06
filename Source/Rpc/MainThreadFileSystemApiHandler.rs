// File: Rpc/MainThreadFileSystemApiHandler.rs
// Defines the RPC handler for filesystem operations requested by the sidecar,
// typically corresponding to the `vscode.workspace.fs` API.

use std::sync::Arc;

use Common::Runtime::AppRuntimeTrait;
use log::{debug, info, trace};
use serde_json::Value;
use tauri::{AppHandle, Manager, State, Wry};

use crate::{
	Handlers::{self, ErrorUtils, WorkspaceFsApi},
	Runtime::AppRuntime,
}; // Assuming WorkspaceFsApi contains the logic

// Note: DTOs for these methods were not explicitly defined in the provided Args
// snippets. The logic will be adapted from `handlers/workspace_fs_api.rs`,
// assuming it can handle raw `Value` parameters or can be refactored to use
// specific DTOs if desired later.

#[derive(Clone)]
pub struct MainThreadFileSystemApiHandler {
	pub ApplicationHandle:AppHandle<Wry>,
	pub Runtime:Arc<AppRuntime>,
}

impl MainThreadFileSystemApiHandler {
	pub fn New(ApplicationHandle:AppHandle<Wry>, Runtime:Arc<AppRuntime>) -> Self {
		Self { ApplicationHandle, Runtime }
	}

	/// Gets file or directory metadata.
	pub async fn Stat(&self, ParametersValue:Value) -> Result<Value, String> {
		debug!("[Rpc FileSystemApiHandler] Stat (DTO flow): Params='{:?}'", ParametersValue);
		WorkspaceFsApi::HandleWorkspaceFsStat(self.Runtime.clone(), ParametersValue).await
	}

	/// Reads the contents of a directory.
	pub async fn ReadDirectory(&self, ParametersValue:Value) -> Result<Value, String> {
		debug!(
			"[Rpc FileSystemApiHandler] ReadDirectory (DTO flow): Params='{:?}'",
			ParametersValue
		);
		WorkspaceFsApi::HandleWorkspaceFsReadDirectory(self.Runtime.clone(), ParametersValue).await
	}

	/// Reads the content of a file.
	pub async fn ReadFile(&self, ParametersValue:Value) -> Result<Value, String> {
		debug!("[Rpc FileSystemApiHandler] ReadFile (DTO flow): Params='{:?}'", ParametersValue);
		WorkspaceFsApi::HandleWorkspaceFsReadFile(self.Runtime.clone(), ParametersValue).await
	}

	/// Writes content to a file.
	pub async fn WriteFile(&self, ParametersValue:Value) -> Result<Value, String> {
		debug!(
			"[Rpc FileSystemApiHandler] WriteFile (DTO flow): Params='{:?}'",
			ParametersValue
		);
		WorkspaceFsApi::HandleWorkspaceFsWriteFile(self.Runtime.clone(), ParametersValue).await
	}

	/// Creates a new directory.
	pub async fn CreateDirectory(&self, ParametersValue:Value) -> Result<Value, String> {
		debug!(
			"[Rpc FileSystemApiHandler] CreateDirectory (DTO flow): Params='{:?}'",
			ParametersValue
		);
		WorkspaceFsApi::HandleWorkspaceFsCreateDirectory(self.Runtime.clone(), ParametersValue).await
	}

	/// Deletes a file or directory.
	pub async fn Delete(&self, ParametersValue:Value) -> Result<Value, String> {
		debug!("[Rpc FileSystemApiHandler] Delete (DTO flow): Params='{:?}'", ParametersValue);
		WorkspaceFsApi::HandleWorkspaceFsDelete(self.Runtime.clone(), ParametersValue).await
	}

	/// Renames a file or directory.
	pub async fn Rename(&self, ParametersValue:Value) -> Result<Value, String> {
		debug!("[Rpc FileSystemApiHandler] Rename (DTO flow): Params='{:?}'", ParametersValue);
		WorkspaceFsApi::HandleWorkspaceFsRename(self.Runtime.clone(), ParametersValue).await
	}

	/// Copies a file or directory.
	pub async fn Copy(&self, ParametersValue:Value) -> Result<Value, String> {
		debug!("[Rpc FileSystemApiHandler] Copy (DTO flow): Params='{:?}'", ParametersValue);
		WorkspaceFsApi::HandleWorkspaceFsCopy(self.Runtime.clone(), ParametersValue).await
	}
}
