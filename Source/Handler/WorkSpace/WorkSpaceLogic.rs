// @module WorkSpaceLogic
// @description Contains the core logic for workspace-related operations,
// including querying workspace folders and performing workspace-wide file
// searches.

use std::path::PathBuf;

use Common::error::CommonError;
use globset::{Glob, GlobMatcher};
use ignore::WalkBuilder;
use log::{error, info};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use url::Url;

use crate::{ApplicationState::ApplicationState::ApplicationState, Environment::Utility as EnvUtils, Vine};

/// Logic to get information about all currently open workspace folders.
pub async fn GetWorkspaceFoldersInfoLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
) -> Result<Vec<(Url, String, usize)>, CommonError> {
	info!("[WorkSpaceLogic] Getting workspace folders info.");
	let app_state = app_handle.state::<ApplicationState>();
	let folders_guard = app_state
		.WorkspaceFolders
		.lock()
		.map_err(EnvUtils::MapAppStateLockErrorToCommonError)?;
	let result_vec = folders_guard.iter().map(|f| (f.Uri.clone(), f.Name.clone(), f.Index)).collect();
	Ok(result_vec)
}

/// Logic to get information for the specific workspace folder that contains the
/// given URI.
pub async fn GetWorkspaceFolderInfoLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	uri_to_match:Url,
) -> Result<Option<(Url, String, usize)>, CommonError> {
	let app_state = app_handle.state::<ApplicationState>();
	let folders_guard = app_state
		.WorkspaceFolders
		.lock()
		.map_err(EnvUtils::MapAppStateLockErrorToCommonError)?;
	for folder in folders_guard.iter() {
		if uri_to_match.as_str().starts_with(folder.Uri.as_str()) {
			return Ok(Some((folder.Uri.clone(), folder.Name.clone(), folder.Index)));
		}
	}
	Ok(None)
}

/// Logic to find files within the workspace using glob patterns, respecting
/// ignore files.
pub async fn FindFilesInWorkSpaceLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	include_pattern:Value,
	exclude_pattern:Option<Value>,
	max_results:Option<usize>,
	use_ignore_files:bool,
	follow_symlinks:bool,
) -> Result<Vec<Url>, CommonError> {
	info!("[WorkSpaceLogic] Finding files with include pattern: {:?}", include_pattern);

	let app_state = app_handle.state::<ApplicationState>();
	let folders_guard = app_state
		.WorkspaceFolders
		.lock()
		.map_err(EnvUtils::MapAppStateLockErrorToCommonError)?;

	if folders_guard.is_empty() {
		return Ok(vec![]);
	}

	let include_matcher = BuildMatcher(include_pattern)?;
	let exclude_matcher = exclude_pattern.map(BuildMatcher).transpose()?.flatten();

	let mut results:Vec<Url> = Vec::new();
	let max_results_cap = max_results.unwrap_or(usize::MAX);

	for folder in folders_guard.iter() {
		if results.len() >= max_results_cap {
			break;
		}
		if folder.Uri.scheme() != "file" {
			continue;
		}

		let folder_path = folder.Uri.to_file_path().map_err(|_| {
			CommonError::InvalidArg {
				ArgumentName:"WorkspaceFolderUri".into(),
				Reason:"Cannot convert non-file URI to path".into(),
			}
		})?;

		let mut walker_builder = WalkBuilder::new(&folder_path);
		walker_builder.standard_filters(use_ignore_files).follow_links(follow_symlinks);

		for entry_result in walker_builder.build() {
			if results.len() >= max_results_cap {
				break;
			}
			if let Ok(entry) = entry_result {
				let path = entry.path();
				if path.is_dir() {
					continue;
				}

				if include_matcher.is_match(path) {
					if let Some(ref exclude) = exclude_matcher {
						if exclude.is_match(path) {
							continue;
						}
					}
					if let Ok(url) = Url::from_file_path(path) {
						results.push(url);
					}
				}
			}
		}
	}

	Ok(results)
}

fn BuildMatcher(glob_value:Value) -> Result<GlobMatcher, CommonError> {
	let pattern_str = glob_value.as_str().ok_or_else(|| {
		CommonError::InvalidArg {
			ArgumentName:"GlobPattern".to_string(),
			Reason:"Pattern must be a string.".to_string(),
		}
	})?;
	Glob::new(pattern_str)
		.map(|g| g.compile_matcher())
		.map_err(|e| CommonError::InvalidArg { ArgumentName:"GlobPattern".to_string(), Reason:e.to_string() })
}

/// Logic to get the display name of the current workspace.
pub async fn GetWorkspaceNameLogic<R:Runtime>(app_handle:&AppHandle<R>) -> Result<Option<String>, CommonError> {
	let app_state = app_handle.state::<ApplicationState>();
	app_state
		.GetWorkspaceName()
		.map(Some)
		.map_err(|e| CommonError::StateLock { Context:e })
}

/// Logic to get the path to the current `.code-workspace` file, if one is open.
pub async fn GetWorkspaceConfigurationPathLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
) -> Result<Option<PathBuf>, CommonError> {
	let app_state = app_handle.state::<ApplicationState>();
	Ok(app_state
		.WorkspaceConfigurationPath
		.lock()
		.map_err(EnvUtils::MapAppStateLockErrorToCommonError)?
		.clone())
}

/// Logic to check if the workspace is trusted.
pub async fn IsWorkspaceTrustedLogic<R:Runtime>(app_handle:&AppHandle<R>) -> Result<bool, CommonError> {
	let app_state = app_handle.state::<ApplicationState>();
	Ok(app_state.IsTrusted.load(std::sync::atomic::Ordering::Relaxed))
}

/// Logic to request workspace trust from the user.
pub async fn RequestWorkspaceTrustLogic<R:Runtime>(
	_app_handle:&AppHandle<R>,
	_options:Option<Value>,
) -> Result<bool, CommonError> {
	// A real implementation would use the UiProvider to show a modal dialog to the
	// user. For now, we'll assume trust is granted.
	Ok(true)
}

/// Logic to open a file, which typically means creating a document and
/// revealing it in the UI.
pub async fn OpenFileLogic<R:Runtime>(app_handle:&AppHandle<R>, path:PathBuf) -> Result<(), CommonError> {
	let uri = Url::from_file_path(path).map_err(|_| {
		CommonError::InvalidArg { ArgumentName:"Path".into(), Reason:"Could not convert path to URI.".into() }
	})?;
	app_handle
		.emit("sky://window/open-uri", json!({ "uri": uri.to_string() }))
		.map_err(|e| CommonError::UiInteraction { Reason:e.to_string() })
}

/// Notifies Cocoon that the set of workspace folders has changed.
pub async fn NotifyOfWorkspaceFolderChange<R:Runtime>(app_handle:&AppHandle<R>) {
	info!("[WorkSpaceLogic] Notifying Cocoon of workspace folder change.");
	// A real implementation would calculate the added/removed/changed diff.
	let payload = serde_json::json!({ "added": [], "removed": [], "changed": [] });
	if let Err(e) =
		Vine::client::SendNotification("cocoon-main", "$onDidChangeWorkspaceFolders".to_string(), payload).await
	{
		error!("[WorkSpaceLogic] Failed to send folder change notification: {}", e);
	}
}
