use std::path::{Path, PathBuf};

use Common::error::CommonError;
use globset::{Glob, GlobMatcher};
use ignore::WalkBuilder;
use log::{error, info};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime};
use url::Url;

/// @module WorkspaceLogic
/// @description Contains the core logic for workspace-related operations,
/// including querying workspace folders and performing workspace-wide file
/// searches.
use crate::{AppState::AppState::AppState, environment::Utils as EnvUtils, vine};

/// Logic to get information about all currently open workspace folders.
pub async fn GetWorkspaceFoldersInfoLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
) -> Result<Vec<(Url, String, usize)>, CommonError> {
	info!("[WorkspaceLogic] Getting workspace folders info.");
	let AppStateInstance = AppHandle.state::<AppState>();
	let FoldersGuard = AppStateInstance
		.WorkspaceFolders
		.lock()
		.map_err(EnvUtils::MapAppStateLockErrorToCommonError)?;
	let ResultVec = FoldersGuard.iter().map(|f| (f.Uri.clone(), f.Name.clone(), f.Index)).collect();
	Ok(ResultVec)
}

/// Logic to find files within the workspace using glob patterns, respecting
/// ignore files.
pub async fn FindFilesInWorkspaceLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
	IncludePattern:Value,
	ExcludePattern:Option<Value>,
	MaxResults:Option<usize>,
	UseIgnoreFiles:bool,
	FollowSymlinks:bool,
) -> Result<Vec<Url>, CommonError> {
	info!("[WorkspaceLogic] Finding files with include pattern: {:?}", IncludePattern);

	let AppStateInstance = AppHandle.state::<AppState>();
	let FoldersGuard = AppStateInstance
		.WorkspaceFolders
		.lock()
		.map_err(EnvUtils::MapAppStateLockErrorToCommonError)?;

	if FoldersGuard.is_empty() {
		return Ok(vec![]);
	}

	let IncludeMatcher = BuildMatcher(IncludePattern)?;
	let ExcludeMatcher = ExcludePattern.map(BuildMatcher).transpose()?.flatten();

	let mut Results:Vec<Url> = Vec::new();
	let MaxResultsCap = MaxResults.unwrap_or(usize::MAX);

	for Folder in FoldersGuard.iter() {
		if Results.len() >= MaxResultsCap {
			break;
		}
		if Folder.Uri.scheme() != "file" {
			continue;
		}

		let FolderPath = PathBuf::from(Folder.Uri.path());
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

				if IncludeMatcher.is_match(Path) {
					if let Some(ref Exclude) = ExcludeMatcher {
						if Exclude.is_match(Path) {
							continue;
						}
					}
					if let Ok(Url) = Url::from_file_path(Path) {
						Results.push(Url);
					}
				}
			}
		}
	}

	Ok(Results)
}

fn BuildMatcher(GlobValue:Value) -> Result<GlobMatcher, CommonError> {
	let PatternStr = GlobValue.as_str().ok_or_else(|| {
		CommonError::InvalidArg {
			ArgumentName:"GlobPattern".to_string(),
			Reason:"Pattern must be a string.".to_string(),
		}
	})?;
	Glob::new(PatternStr)
		.map(|g| g.compile_matcher())
		.map_err(|e| CommonError::InvalidArg { ArgumentName:"GlobPattern".to_string(), Reason:e.to_string() })
}

/// Logic to get the path to the current `.code-workspace` file, if one is open.
pub async fn GetWorkspaceConfigurationPathLogic<R:Runtime>(
	AppHandle:&AppHandle<R>,
) -> Result<Option<PathBuf>, CommonError> {
	let AppStateInstance = AppHandle.state::<AppState>();
	Ok(AppStateInstance
		.WorkspaceConfigurationPath
		.lock()
		.map_err(EnvUtils::MapAppStateLockErrorToCommonError)?
		.clone())
}

/// Notifies Cocoon that the set of workspace folders has changed.
pub async fn NotifyOfWorkspaceFolderChange<R:Runtime>(AppHandle:&AppHandle<R>) {
	info!("[WorkspaceLogic] Notifying Cocoon of workspace folder change.");
	let Payload = serde_json::json!({ "added": [], "removed": [], "changed": [] }); // A real impl would have the diff
	if let Err(e) =
		vine::client::SendNotification("cocoon-main", "$onDidChangeWorkspaceFolders".to_string(), Payload).await
	{
		error!("[WorkspaceLogic] Failed to send folder change notification: {}", e);
	}
}
