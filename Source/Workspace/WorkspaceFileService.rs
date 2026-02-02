// File: Mountain/Source/Workspace/WorkspaceFileService.rs
// Role: Contains logic for parsing and handling `.code-workspace` files.
// Responsibilities:
//   - Define the structure for deserializing `.code-workspace` JSON.
//   - Parse the file content, resolve relative folder paths to absolute URIs,

//     and construct a list of `WorkspaceFolderStateDTO`s representing the
//     workspace.

//! # Workspace File Service
//!
//! Contains logic for parsing and handling `.code-workspace` files.

use std::path::Path;

use CommonLibrary::Error::CommonError::CommonError;
use serde::Deserialize;
use url::Url;

use crate::ApplicationState::DTO::WorkspaceFolderStateDTO::WorkspaceFolderStateDTO;

#[derive(Deserialize, Debug)]
struct WorkspaceFile {
	folders:Vec<WorkspaceFolderEntry>,
	// Can also contain 'settings', 'extensions', etc.
}

#[derive(Deserialize, Debug)]
struct WorkspaceFolderEntry {
	path:String,
}

/// Parses a `.code-workspace` file content and resolves the folder paths.
///
/// # Parameters
/// * `WorkspaceFilePath`: The absolute path to the `.code-workspace` file.
/// * `FileContent`: The raw string content of the file.
///
/// # Returns
/// A `Result` containing a vector of `WorkspaceFolderStateDTO`s.
pub fn ParseWorkspaceFile(
	WorkspaceFilePath:&Path,

	FileContent:&str,
) -> Result<Vec<WorkspaceFolderStateDTO>, CommonError> {
	let Parsed:WorkspaceFile = serde_json::from_str(FileContent)
		.map_err(|Error| CommonError::SerializationError { Description:Error.to_string() })?;

	let WorkspaceFileDirectory = WorkspaceFilePath.parent().ok_or_else(|| {
		CommonError::FileSystemIO {
			Path:WorkspaceFilePath.to_path_buf(),

			Description:"Cannot get parent directory of workspace file".to_string(),
		}
	})?;

	let Folders:Result<Vec<WorkspaceFolderStateDTO>, CommonError> = Parsed
		.folders
		.into_iter()
		.enumerate()
		.map(|(Index, Entry)| {
			let FolderPath = WorkspaceFileDirectory.join(Entry.path);

			let CanonicalPath = FolderPath
				.canonicalize()
				.map_err(|_| CommonError::FileSystemNotFound(FolderPath.clone()))?;

			let FolderURI = Url::from_directory_path(&CanonicalPath).map_err(|_| {
				CommonError::InvalidArgument {
					ArgumentName:"path".into(),

					Reason:format!("Could not convert path '{}' to URL", CanonicalPath.display()),
				}
			})?;

			let Name = CanonicalPath
				.file_name()
				.and_then(|n| n.to_str())
				.unwrap_or("untitled-folder")
				.to_string();

			Ok(WorkspaceFolderStateDTO { URI:FolderURI, Name, Index })
		})
		.collect();

	Folders
}
