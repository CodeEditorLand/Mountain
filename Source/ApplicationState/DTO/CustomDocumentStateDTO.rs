//! # CustomDocumentStateDTO
//!
//! # RESPONSIBILITY
//! - Data transfer object for custom editor document state
//! - Serializable format for gRPC/IPC transmission
//! - Used by Mountain to track custom document lifecycle
//!
//! # FIELDS
//! - URI: Resource identifier for the document
//! - ViewType: Custom editor type identifier
//! - SideCarIdentifier: Sidecar process hosting the provider
//! - IsEditable: User edit permission flag
//! - BackupIdentifier: Optional backup file reference
//! - Edits: Version tracking and edit history map

#![allow(non_snake_case, non_camel_case_types)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::ApplicationState::Internal::URLSerializationHelper;

/// Maximum length for ViewType string to prevent allocation attacks
const MAX_VIEW_TYPE_LENGTH: usize = 256;

/// Maximum length for SideCarIdentifier string
const MAX_SIDECAR_IDENTIFIER_LENGTH: usize = 128;

/// Maximum number of edits to track per document
const MAX_EDITS_PER_DOCUMENT: usize = 1000;

/// A struct that holds the state for a document being handled by a custom
/// editor. This is stored in `ApplicationState` to track the lifecycle of
/// custom documents.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct CustomDocumentStateDTO {
	/// The URI of the document resource being edited.
	#[serde(with = "URLSerializationHelper")]
	pub URI:Url,

	/// The view type of the custom editor responsible for this document.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub ViewType:String,

	/// The identifier of the sidecar process where the custom editor provider
	/// lives.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub SideCarIdentifier:String,

	/// A flag indicating if the document is currently editable by the user.
	pub IsEditable:bool,

	/// An optional identifier for a backup copy of the file's content.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub BackupIdentifier:Option<String>,

	/// A map to store edit history or other versioning information.
	/// In a real implementation, this might hold a more structured edit type.
	#[serde(skip_serializing_if = "HashMap::is_empty")]
	pub Edits:HashMap<u32, serde_json::Value>,
}

impl CustomDocumentStateDTO {
	/// Creates a new CustomDocumentStateDTO with validation.
	///
	/// # Arguments
	/// * `URI` - The document resource URI
	/// * `ViewType` - The custom editor type identifier
	/// * `SideCarIdentifier` - The sidecar process identifier
	/// * `IsEditable` - Whether the document is user-editable
	///
	/// # Returns
	/// Result containing the DTO or an error if validation fails
	pub fn New(URI:Url, ViewType:String, SideCarIdentifier:String, IsEditable:bool) -> Result<Self, String> {
		// Validate ViewType length
		if ViewType.len() > MAX_VIEW_TYPE_LENGTH {
			return Err(format!("ViewType exceeds maximum length of {} bytes", MAX_VIEW_TYPE_LENGTH));
		}

		// Validate SideCarIdentifier length
		if SideCarIdentifier.len() > MAX_SIDECAR_IDENTIFIER_LENGTH {
			return Err(format!("SideCarIdentifier exceeds maximum length of {} bytes", MAX_SIDECAR_IDENTIFIER_LENGTH));
		}

		// Ensure URI is not empty
		if URI.as_str().is_empty() {
			return Err("URI cannot be empty".to_string());
		}

		Ok(Self {
			URI,
			ViewType,
			SideCarIdentifier,
			IsEditable,
			BackupIdentifier: None,
			Edits: HashMap::new(),
		})
	}

	/// Adds an edit entry to the edits map with bounds checking.
	///
	/// # Arguments
	/// * `EditID` - The edit identifier
	/// * `EditData` - The edit data
	///
	/// # Returns
	/// Result indicating success or failure if map is full
	pub fn AddEdit(&mut self, EditID:u32, EditData:serde_json::Value) -> Result<(), String> {
		if self.Edits.len() >= MAX_EDITS_PER_DOCUMENT {
			return Err(format!("Maximum edit limit of {} reached for document", MAX_EDITS_PER_DOCUMENT));
		}

		self.Edits.insert(EditID, EditData);
		Ok(())
	}

	/// Clears all edit history for this document.
	pub fn ClearEdits(&mut self) {
		self.Edits.clear();
	}

	/// Returns the count of edits tracked for this document.
	pub fn GetEditCount(&self) -> usize {
		self.Edits.len()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_creation_success() {
		let URI = Url::parse("file:///test/document.md").unwrap();
		let dto = CustomDocumentStateDTO::New(
			URI.clone(),
			"markdown.editor".to_string(),
			"sidecar-123".to_string(),
			true
		);
		assert!(dto.is_ok());
		assert_eq!(dto.unwrap().ViewType, "markdown.editor");
	}

	#[test]
	fn test_invalid_view_type_length() {
		let URI = Url::parse("file:///test/document.md").unwrap();
		let LongViewType = "a".repeat(257);
		let dto = CustomDocumentStateDTO::New(
			URI,
			LongViewType,
			"sidecar-123".to_string(),
			true
		);
		assert!(dto.is_err());
	}
}
