//! Parse a `.code-workspace` file's content and resolve every folder path to a
//! `file://` URI.
//!
//! ## Status
//!
//! Zero callers as of 2026-05-02. The parser exists in case Mountain
//! ever owns `.code-workspace` ingestion (currently delegated through Wind).
//! Remove if the boundary stays on the Wind side.

use std::path::Path;

use CommonLibrary::Error::CommonError::CommonError;
use url::Url;
use ::Cache::PathCanon::Canonicalize;

use crate::{
	ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	Workspace::WorkspaceFileService::WorkspaceFile,
};

pub fn Fn(WorkspaceFilePath:&Path, FileContent:&str) -> Result<Vec<WorkspaceFolderStateDTO>, CommonError> {
	let Parsed:WorkspaceFile::Struct = serde_json::from_str(FileContent)
		.map_err(|Error| CommonError::SerializationError { Description:Error.to_string() })?;

	let WorkspaceFileDirectory = WorkspaceFilePath.parent().ok_or_else(|| {
		CommonError::FileSystemIO {
			Path:WorkspaceFilePath.to_path_buf(),
			Description:"Cannot get parent directory of workspace file".to_string(),
		}
	})?;

	Parsed
		.folders
		.into_iter()
		.enumerate()
		.map(|(Index, Entry)| {
			let FolderPath = WorkspaceFileDirectory.join(Entry.path);

			let CanonicalPath =
				Canonicalize::Fn(&FolderPath).map_err(|_| CommonError::FileSystemNotFound(FolderPath.clone()))?;

			let FolderURI = Url::from_directory_path(&CanonicalPath).map_err(|_| {
				CommonError::InvalidArgument {
					ArgumentName:"path".into(),
					Reason:format!("Could not convert path '{}' to URL", CanonicalPath.display()),
				}
			})?;

			let Name = CanonicalPath
				.file_name()
				.and_then(|N| N.to_str())
				.unwrap_or("untitled-folder")
				.to_string();

			Ok(WorkspaceFolderStateDTO { URI:FolderURI, Name, Index })
		})
		.collect()
}
