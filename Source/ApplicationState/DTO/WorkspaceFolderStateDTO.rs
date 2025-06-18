// @module WorkspaceFolderStateDTO
// @description Defines the Data Transfer Object for storing the state of a
// single workspace folder.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};
use url::Url;

use super::super::Internal::UrlSerdeHelper;

// Represents a single folder that is part of the current workspace.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct WorkspaceFolderStateDTO {
	// The URI of the folder.
	#[serde(with = "UrlSerdeHelper")]
	pub Uri:Url,
	// The display name of the folder.
	pub Name:String,
	// The zero-based index of the folder in the workspace.
	pub Index:usize,
}
