//! # MarkerDataDTO
//!
//! Defines the Data Transfer Object for a single diagnostic marker (e.g., an
//! error or warning).

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents a single diagnostic marker, such as a compiler error or a linter
/// warning. This structure is compatible with VS Code's `IMarkerData`
/// interface and is used by the Diagnostic service.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct MarkerDataDTO {
	pub Severity:u32,
	pub Message:String,
	pub StartLineNumber:u32,
	pub StartColumn:u32,
	pub EndLineNumber:u32,
	pub EndColumn:u32,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub Source:Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub Code:Option<Value>, // Can be string or { value: string, target: URI }

	#[serde(skip_serializing_if = "Option::is_none")]
	pub ModelVersionIdentifier:Option<u64>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub RelatedInformation:Option<Value>, // Vec<RelatedInformationDTO>

	#[serde(skip_serializing_if = "Option::is_none")]
	pub Tags:Option<Vec<u32>>, // Corresponds to MarkerTag enum
}
