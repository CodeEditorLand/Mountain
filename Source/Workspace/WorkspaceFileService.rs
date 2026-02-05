//! # WorkspaceFileService (Workspace)
//!
//! RESPONSIBILITIES:
//! - Parses `.code-workspace` configuration files (VSCode multi-root workspace format)
//! - Resolves relative folder paths to absolute filesystem paths
//! - Converts parsed workspace folders into `WorkspaceFolderStateDTO` instances
//! - Handles path canonicalization and URI conversion
//!
//! ARCHITECTURAL ROLE:
//! - Utility module for workspace configuration management
//! - Used by [`MountainEnvironment`](crate::Environment::MountainEnvironment) during
//!   workspace initialization and configuration loading
//! - Integrates with [`ApplicationState`](crate::ApplicationState::ApplicationState)
//!   for workspace folder state management
//!
//! FILE FORMAT:
//! - Expects JSON format conforming to VSCode `.code-workspace` schema
//! - Top-level object contains `folders` array (can also have `settings`, `extensions`)
//! - Each folder entry has at least a `path` field (relative to workspace file)
//! - Example:
//!   ```json
//!   {
//!     "folders": [
//!       { "path": "." },
//!       { "path": "../other-project" }
//!     ]
//!   }
//!   ```
//!
//! PARSING FLOW:
//! 1. Read `.code-workspace` file content as string
//! 2. Deserialize JSON into `WorkspaceFile` struct (using serde)
//! 3. Determine workspace file's parent directory (base for relative paths)
//! 4. For each folder entry:
//!    - Join relative path with base directory
//!    - Canonicalize to resolve symlinks and relative segments (`..`, `.`)
//!    - Convert absolute path to `file://` URI via `Url::from_directory_path`
//!    - Extract folder name from path (fallback to "untitled-folder")
//!    - Assign sequential index based on declaration order
//! 5. Return `Vec<WorkspaceFolderStateDTO>` with all resolved folders
//!
//! ERROR HANDLING:
//! - Returns [`CommonError`](CommonLibrary::Error::CommonError) on any failure
//! - JSON deserialization errors → `SerializationError`
//! - Missing parent directory → `FileSystemIO`
//! - Path canonicalization failure → `FileSystemNotFound`
//! - URI conversion failure → `InvalidArgument`
//!
//! PERFORMANCE:
//! - Synchronous function but should be called from async context
//! - Each folder path undergoes I/O: join + canonicalize (can be slow on network drives)
//! - Consider caching parsed workspace files if frequently accessed
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/workspaces/common/workspace.ts` - workspace file format
//! - `vs/workbench/services/workspace/browser/workspaceService.ts` - workspace service
//! - `vs/platform/workspace/common/workspace.ts` - workspace interfaces
//!
//! TODO:
//! - Add validation for workspace file schema version
//! - Support workspace file `settings` section (merge into configuration)
//! - Parse workspace `extensions` section (recommend extensions)
//! - Add support for workspace file glob patterns in folder paths
//! - Implement workspace file change watching (reload on external edits)
//! - Add workspace file templates for common multi-root configurations
//! - Support workspace file encryption for sensitive paths
//! - Handle workspace files on remote filesystems (SSH, containers)
//! - Add workspace file migration tools (format upgrades)
//! - Implement workspace file validation and linting
//! - Support workspace file includes (composite workspaces)
//!
//! MODULE CONTENTS:
//! - Structs: `WorkspaceFile`, `WorkspaceFolderEntry` (serde deserialization)
//! - Function: `ParseWorkspaceFile` - main entry point
//! - Data type: [`WorkspaceFolderStateDTO`](crate::ApplicationState::DTO::WorkspaceFolderStateDTO)

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
