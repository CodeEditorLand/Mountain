//! # ExtensionDescriptionStateDTO
//!
//! Defines the Data Transfer Object for storing the state of a single scanned
//! extension, based on its `package.json` manifest.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the deserialized content of an extension's `package.json` file,

/// augmented with location information and other metadata.
///
/// This is stored in `ApplicationState` to provide the extension host with the
/// list of available extensions and their capabilities.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ExtensionDescriptionStateDTO {
	// --- Core Metadata ---
	// DTO: { value: string, uuid?: string }
	pub Identifier:Value,

	pub Name:String,

	pub Version:String,

	pub Publisher:String,

	// DTO: { vscode: string }
	pub Engines:Value,

	// --- Entry Points ---
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Main:Option<String>,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub Browser:Option<String>,

	// --- Type & Flags ---
	#[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
	pub ModuleType:Option<String>,

	#[serde(default)]
	pub IsBuiltin:bool,

	#[serde(default)]
	pub IsUnderDevelopment:bool,

	// --- Location & Activation ---
	// Serialized UriComponents DTO
	pub ExtensionLocation:Value,

	#[serde(skip_serializing_if = "Option::is_none")]
	pub ActivationEvents:Option<Vec<String>>,

	// --- Contributions ---
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Contributes:Option<Value>,
}
