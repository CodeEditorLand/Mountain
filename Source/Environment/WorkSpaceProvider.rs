//! # WorkSpaceProvider Implementation
//!
//! Implements the `WorkSpaceProvider` and `WorkSpaceEditApplier` traits for
//! the `MountainEnvironment`. This provider contains the core logic for
//! workspace-related operations, including querying workspace folders and
//! performing workspace-wide file searches.

use std::path::PathBuf;

use Common::{
	DTO::WorkSpaceEditDTO::WorkSpaceEditDTO,
	Error::CommonError::CommonError,
	WorkSpace::{WorkSpaceEditApplier::WorkSpaceEditApplier, WorkSpaceProvider::WorkSpaceProvider},
};
use async_trait::async_trait;
use globset::{Glob, GlobMatcher};
use ignore::WalkBuilder;
use log::{info, warn};
use serde_json::Value;
use tauri::Emitter;
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
		let ResultVector = FoldersGuard.iter().map(|f| (f.URI.clone(), f.Name.clone(), f.Index)).collect();
		Ok(ResultVector)
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
		// This logic is complex and better suited inside ApplicationState.
		// For now, it's a stub.
		warn!("[WorkSpaceProvider] GetWorkSpaceName is a stub.");
		Ok(Some("Untitled WorkSpace".to_string()))
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
		// A real implementation would use the UserInterfaceProvider.
		warn!("[WorkSpaceProvider] RequestWorkSpaceTrust is not implemented; defaulting to trusted.");
		Ok(true)
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
				Err(_) => continue, // Skip non-file URIs
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
					if let Some(ref include) = IncludeMatcher {
						if include.is_match(Path) {
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
			}
		}
		Ok(Results)
	}

	/// Requests that the host application open a file in an editor.
	async fn OpenFile(&self, Path:PathBuf) -> Result<(), CommonError> {
		let URI = Url::from_file_path(Path).map_err(|_| {
			CommonError::InvalidArgument { ArgumentName:"Path".into(), Reason:"Could not convert path to URI.".into() }
		})?;
		self.ApplicationHandle
			.emit("sky://window/open-uri", serde_json::json!({ "uri": URI.to_string() }))
			.map_err(|e| CommonError::UserInterfaceInteraction { Reason:e.to_string() })
	}
}

fn BuildGlobMatcher(GlobValue:Value) -> Result<Option<GlobMatcher>, CommonError> {
	GlobValue
		.as_str()
		.map(|Pattern| {
			Glob::new(Pattern).map(|g| g.compile_matcher()).map_err(|e| {
				CommonError::InvalidArgument { ArgumentName:"GlobPattern".to_string(), Reason:e.to_string() }
			})
		})
		.transpose()
}

#[async_trait]
impl WorkSpaceEditApplier for MountainEnvironment {
	async fn ApplyWorkSpaceEdit(&self, _EditDTO:WorkSpaceEditDTO) -> Result<bool, CommonError> {
		warn!("[WorkSpaceProvider] ApplyWorkSpaceEdit is not implemented.");
		// A full implementation would use DocumentProvider and FileSystemWriter
		// effects.
		Err(CommonError::NotImplemented { FeatureName:"ApplyWorkSpaceEdit".into() })
	}
}
