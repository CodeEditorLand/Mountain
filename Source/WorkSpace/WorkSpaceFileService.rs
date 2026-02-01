// File: Mountain/Source/WorkSpace/WorkSpaceFileService.rs
// Role: Contains logic for parsing and handling `.code-workspace` files.
// Responsibilities:
//   - Define the structure for deserializing `.code-workspace` JSON.
//   - Parse the file content, resolve relative folder paths to absolute URIs,

//     and construct a list of `WorkSpaceFolderStateDTO`s representing the
//     workspace.

//! # WorkSpace File Service
//!
//! Contains logic for parsing and handling `.code-workspace` files.

use std::path::Path;

use CommonLibrary::Error::CommonError::CommonError;
use serde::Deserialize;
use url::Url;

use crate::ApplicationState::DTO::WorkSpaceFolderStateDTO::WorkSpaceFolderStateDTO;

#[derive(Deserialize, Debug)]
struct WorkSpaceFile {
	folders:Vec<WorkSpaceFolderEntry>,
	// Can also contain 'settings', 'extensions', etc.
}

#[derive(Deserialize, Debug)]
struct WorkSpaceFolderEntry {
	path:String,
}

/// Parses a `.code-workspace` file content and resolves the folder paths.
///
/// # Parameters
/// * `WorkSpaceFilePath`: The absolute path to the `.code-workspace` file.
/// * `FileContent`: The raw string content of the file.
///
/// # Returns
/// A `Result` containing a vector of `WorkSpaceFolderStateDTO`s.
pub fn ParseWorkSpaceFile(
	WorkSpaceFilePath:&Path,

	FileContent:&str,
) -> Result<Vec<WorkSpaceFolderStateDTO>, CommonError> {
	let Parsed:WorkSpaceFile = serde_json::from_str(FileContent)
		.map_err(|Error| CommonError::SerializationError { Description:Error.to_string() })?;

	let WorkSpaceFileDirectory = WorkSpaceFilePath.parent().ok_or_else(|| {
		CommonError::FileSystemIO {
			Path:WorkSpaceFilePath.to_path_buf(),

			Description:"Cannot get parent directory of workspace file".to_string(),
		}
	})?;

	let Folders:Result<Vec<WorkSpaceFolderStateDTO>, CommonError> = Parsed
		.folders
		.into_iter()
		.enumerate()
		.map(|(Index, Entry)| {
			let FolderPath = WorkSpaceFileDirectory.join(Entry.path);

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

			Ok(WorkSpaceFolderStateDTO { URI:FolderURI, Name, Index })
		})
		.collect();

	Folders
}
