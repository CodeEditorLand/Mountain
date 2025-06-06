// File: AppState/WorkspaceFolderState.rs
// Defines the data structure for representing a single folder within a
// workspace.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};
use url::Url;

use crate::AppState::Internal::UrlSerdeHelper; // Assuming this path based on previous files

/// Represents the state of a single workspace folder.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct WorkspaceFolderState {
	// The URI of the folder, serialized to/from a string.
	#[serde(with = "UrlSerdeHelper")]
	pub Uri:Url,
	// The display name of the folder.
	pub Name:String,
	// The zero-based index of the folder within the workspace.
	pub Index:usize,
}
