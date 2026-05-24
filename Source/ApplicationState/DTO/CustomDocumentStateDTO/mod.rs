pub mod New;
pub mod AddEdit;
pub mod ClearEdits;
pub mod GetEditCount;

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::Utility::Serialization::URLSerializationHelper;

/// A struct that holds the state for a document being handled by a custom
/// editor. This is stored in `ApplicationState` to track the lifecycle of
/// custom documents.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Struct {
	/// The URI of the document resource being edited.
	#[serde(rename = "uri", with = "URLSerializationHelper")]
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
