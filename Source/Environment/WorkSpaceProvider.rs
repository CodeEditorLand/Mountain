// File: Mountain/Source/Environment/WorkSpaceProvider.rs
// Role: Implements `WorkSpaceProvider` and `WorkSpaceEditApplier` traits.
// Responsibilities:
//   - Core logic for workspace-related operations.
//   - Querying workspace folders, finding files, and applying workspace edits.
//   - Orchestrating the opening of files, including routing to custom editors.

//! # WorkSpaceProvider Implementation
//!
//! Implements the `WorkSpaceProvider` and `WorkSpaceEditApplier` traits for
//! the `MountainEnvironment`. This provider contains the core logic for
//! workspace-related operations, including querying workspace folders and
//! performing workspace-wide file searches.

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
		Ok(self.ApplicationState.GetWorkSpaceIdentifier().ok())
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

					let is_match = IncludeMatcher.as_ref().map_or(true, |g| g.is_match(Path));

					if !is_match {
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

		// TODO: A full implementation would check a registry of custom editor providers
		// based on the file's glob pattern.
		let custom_editor_view_type:Option<String> = None;

		if let Some(view_type) = custom_editor_view_type {
			info!(
				"[WorkSpaceProvider] Found custom editor '{}' for file '{}'",
				view_type,
				Path.display()
			);

			let webview_provider:Arc<dyn WebViewProvider> = self.Require();

			let handle = webview_provider
				.CreateWebViewPanel(
					json!({ "id": "placeholder.extension" }),
					view_type.clone(),
					Path.file_name().unwrap().to_string_lossy().to_string(),
					json!({ "viewColumn": -1 }),
					json!({}),
					json!({ "enableScripts": true }),
				)
				.await?;

			let custom_editor_provider:Arc<dyn CustomEditorProvider> = self.Require();

			custom_editor_provider.ResolveCustomEditor(view_type, URI, handle).await?;

			return Ok(());
		}

		info!(
			"[WorkSpaceProvider] No custom editor found. Opening '{}' as text.",
			Path.display()
		);

		let uri_components = json!({ "external": URI.to_string(), "$mid": 1 });

		let doc_provider:Arc<dyn DocumentProvider> = self.Require();

		doc_provider.OpenDocument(uri_components, None, None).await?;

		Ok(())
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
	async fn ApplyWorkSpaceEdit(&self, EditDTO:WorkSpaceEditDTO) -> Result<bool, CommonError> {
		let DocProvider:Arc<dyn DocumentProvider> = self.Require();

		for (URIValue, Edits) in EditDTO.Edits {
			let URI = serde_json::from_value::<Url>(URIValue)?;

			let Document = {
				let Guard = self.ApplicationState.OpenDocuments.lock().unwrap();

				Guard.get(URI.as_str()).cloned()
			};

			if let Some(Doc) = Document {
				let NewVersionID = Doc.Version + 1;

				DocProvider
					.ApplyDocumentChanges(URI.clone(), NewVersionID.into(), json!(Edits), true, false, false)
					.await?;
			} else {
				warn!("[WorkSpaceProvider] Attempted to apply edit to non-open document: {}", URI);
			}
		}

		Ok(true)
	}
}
