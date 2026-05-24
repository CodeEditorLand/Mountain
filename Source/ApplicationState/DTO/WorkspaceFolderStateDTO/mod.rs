pub mod New;
pub mod UpdateName;
pub mod GetDisplayName;
pub mod IsRoot;
pub mod FromPath;

use serde::{Deserialize, Serialize};
use url::Url;
use CommonLibrary::Utility::Serialization::URLSerializationHelper;

/// Represents a single folder that is part of the current workspace.
/// Compatible with VS Code's WorkspaceFolder interface.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Struct {
	/// The URI of the folder.
	#[serde(rename = "uri", with = "URLSerializationHelper")]
	pub URI:Url,

	/// The display name of the folder.
	#[serde(skip_serializing_if = "String::is_empty")]
	pub Name:String,

	/// The zero-based index of the folder in the workspace.
	pub Index:usize,
}
