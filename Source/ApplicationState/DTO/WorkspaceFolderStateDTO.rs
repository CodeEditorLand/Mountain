//! # WorkspaceFolderStateDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for workspace folder state
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to track workspace folder configuration
//!
//! # FIELDS
//! - URI: Folder resource URI
//! - Name: Display name
//! - Index: Zero-based position in workspace
use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::Utility::Serialization::URLSerializationHelper;

/// Maximum folder name length
const MAX_FOLDER_NAME_LENGTH:usize = 256;

/// Maximum number of folders in a workspace
const MAX_WORKSPACE_FOLDERS:usize = 100;

/// a single folder that is part of the current workspace.
/// Compatible with VS Code's WorkspaceFolder interface.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFolderStateDTO {
	/// The URI of the folder.
	#[serde(rename = "uri", with = "URLSerializationHelper")]
	pub URI:Url,

	/// The display name of the folder.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub Name:String,

	/// The zero-based index of the folder in the workspace.
	pub Index:usize,
}

impl WorkspaceFolderStateDTO {
	/// Creates a new WorkspaceFolderStateDTO with validation.
	/// # Arguments
	/// * `URI` - Folder URI
	/// * `Name` - Display name
	/// * `Index` - Zero-based index in workspace
	/// # Returns
	/// Result containing the DTO or validation error
	pub fn New(URI:Url, Name:String, Index:usize) -> Result<Self, String> {
		// Validate URI is not empty
		if URI.as_str().is_empty() {
			return Err("URI cannot be empty".to_string());
		}

		// Validate name length
		if Name.len() > MAX_FOLDER_NAME_LENGTH {
			return Err(format!(
				"Folder name exceeds maximum length of {} bytes",
				MAX_FOLDER_NAME_LENGTH
			));
		}

		// Validate index range
		if Index >= MAX_WORKSPACE_FOLDERS {
			return Err(format!(
				"Folder index {} exceeds maximum workspace folders count of {}",
				Index, MAX_WORKSPACE_FOLDERS
			));
		}

		Ok(Self { URI, Name, Index })
	}

	/// Updates the name with validation.
	/// # Arguments
	/// * `Name` - New display name
	/// # Returns
	/// Result indicating success or error if name too long
	pub fn UpdateName(&mut self, Name:String) -> Result<(), String> {
		if Name.len() > MAX_FOLDER_NAME_LENGTH {
			return Err(format!(
				"Folder name exceeds maximum length of {} bytes",
				MAX_FOLDER_NAME_LENGTH
			));
		}

		self.Name = Name;

		Ok(())
	}

	/// Gets the folder name as a human-readable string.
	/// Returns the name if present, otherwise extracts from URI.
	pub fn GetDisplayName(&self) -> String {
		if !self.Name.is_empty() {
			self.Name.clone()
		} else {
			// Extract folder name from URI
			self.URI
				.path_segments()
				.and_then(|Segments| Segments.last())
				.unwrap_or("Untitled")
				.to_string()
		}
	}

	/// Checks if this is the root folder (index 0).
	pub fn IsRoot(&self) -> bool { self.Index == 0 }

	/// Creates a new instance from a file path URI.
	/// # Arguments
	/// * `FolderPath` - Folder path as string
	/// * `Index` - Folder index
	/// # Returns
	/// Result containing the DTO or validation error
	pub fn FromPath(FolderPath:&str, Index:usize) -> Result<Self, String> {
		let URI = Url::parse(FolderPath).map_err(|Error| format!("Invalid folder path: {}", Error))?;

		// Check if the URI represents a directory by checking if it ends with a slash
		// or if the file path exists and is a directory
		let IsDirectory =
			URI.path().ends_with('/') || (URI.scheme() == "file" && URI.to_file_path().map_or(false, |p| p.is_dir()));

		if !IsDirectory {
			return Err("URI does not represent a directory".to_string());
		}

		let Name = Self::ExtractFolderName(&URI);

		Self::New(URI, Name, Index)
	}

	/// Extracts the folder name from a URI.
	fn ExtractFolderName(URI:&Url) -> String {
		URI.path_segments()
			.and_then(|Segments| Segments.last())
			.map(String::from)
			.unwrap_or_else(|| "Untitled".to_string())
	}
}

#[cfg(test)]
mod tests {

	use super::*;

	#[test]
	fn test_creation_success() {
		let URI = Url::parse("file:///workspace/project").unwrap();

		let dto = WorkspaceFolderStateDTO::New(URI.clone(), "project".to_string(), 0);

		assert!(dto.is_ok());

		assert_eq!(dto.unwrap().Name, "project");
	}

	#[test]
	fn test_invalid_name_length() {
		let URI = Url::parse("file:///workspace/project").unwrap();

		let LongName = "a".repeat(257);

		let dto = WorkspaceFolderStateDTO::New(URI, LongName, 0);

		assert!(dto.is_err());
	}

	#[test]
	fn test_invalid_index() {
		let URI = Url::parse("file:///workspace/project").unwrap();

		let dto = WorkspaceFolderStateDTO::New(URI, "project".to_string(), 100);

		assert!(dto.is_err());
	}
}
