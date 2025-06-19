//! # TreeViewStateDTO
//!
//! Defines the Data Transfer Object for storing the state of a single
//! registered tree view.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};

/// Holds the static options for a tree view instance that has been registered
/// by an extension. This is stored in `ApplicationState` to track active tree
/// views.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TreeViewStateDTO {
	pub ViewIdentifier:String,
	#[serde(default)]
	pub CanSelectMany:bool,
	#[serde(default)]
	pub HasHandleDrag:bool,
	#[serde(default)]
	pub HasHandleDrop:bool,
}
