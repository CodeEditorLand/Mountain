// @module TreeViewStateDTO
// @description Defines the Data Transfer Object for storing the state of a
// single registered tree view.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};

// Holds the static options for a tree view instance that has been registered
// by an extension.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TreeViewStateDTO {
	pub ViewId:String,
	#[serde(default)]
	pub CanSelectMany:bool,
	#[serde(default)]
	pub HasHandleDrag:bool,
	#[serde(default)]
	pub HasHandleDrop:bool,
}
