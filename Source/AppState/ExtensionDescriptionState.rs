// File: AppState/ExtensionDescriptionState.rs
// Defines the data structure for representing the metadata of a single scanned
// extension, parsed from its `package.json` manifest.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the essential metadata of an extension.
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ExtensionDescriptionState {
	// A JSON object containing the extension's identifier, e.g., `{ "value": "publisher.name", "uuid": "..." }`.
	pub Identifier:Value,
	// The human-readable name of the extension.
	pub Name:String,
	// The version string of the extension (e.g., "1.0.0").
	pub Version:String,
	// The name of the extension's publisher.
	pub Publisher:String,
	// A JSON object specifying the required engine versions (e.g., `{ "vscode": "^1.80.0" }`).
	pub Engines:Value,
	// Optional. The path to the main JavaScript entry file for a Node.js-based extension host.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Main:Option<String>,
	// Optional. The path to the main JavaScript entry file for a web-based extension host.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Browser:Option<String>,
	// Optional. Specifies the module type, e.g., "commonjs" or "module" (for ES Modules).
	#[serde(rename = "Type", skip_serializing_if = "Option::is_none")]
	pub ModuleType:Option<String>,
	// True if the extension is bundled with the application.
	#[serde(default)]
	pub IsBuiltin:bool,
	// True if the extension is being run in a development context.
	#[serde(default)]
	pub IsUnderDevelopment:bool,
	// A UriComponents DTO representing the filesystem location of the extension.
	pub ExtensionLocation:Value,
	// Optional. A list of events that will trigger the activation of the extension.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ActivationEvents:Option<Vec<String>>,
	// Optional. A JSON object representing the contributions of the extension
	// (e.g., commands, languages, themes).
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Contributes:Option<Value>,
}
