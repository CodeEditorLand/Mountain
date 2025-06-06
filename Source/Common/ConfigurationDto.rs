
// Defines Data Transfer Objects (DTOs) related to application configuration.
// These structs are used for serialization and communication between different
// parts of the system, such as between the backend and the sidecar.

#![allow(non_snake_case, non_camel_case_types)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Specifies the target scope for a configuration update.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationTarget {
	UserLocal = 1,
	User = 2,
	Workspace = 3,
	WorkspaceFolder = 4,
	Default = 5,
	Memory = 6,
	Policy = 7,
}

/// Defines the scope of a configuration setting.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ConfigurationScope {
	Application = 1,
	Machine = 2,
	Window = 3,
	Resource = 4,
	LanguageDefined = 5,
	MachineOverridable = 6,
}

/// Represents overrides for retrieving configuration values, such as for a
/// specific resource or language.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct IConfigurationOverrides {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Resource:Option<Value>, // UriComponents DTO
	#[serde(skip_serializing_if = "Option::is_none")]
	pub OverrideIdentifier:Option<String>, // Language ID
}

/// Contains all configuration data needed to initialize the extension host.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct IConfigurationInitDataDto {
	pub Effective:Value,
	pub Defaults:Value,
	pub User:Value,
	pub Workspace:Value,
	pub Folders:Value,
	pub Memory:Value,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Policy:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub ConfigurationScopes:Option<Vec<(String, Value)>>,
}

/// Represents the detailed inspection results for a single configuration key.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct InspectResultData {
	#[serde(alias = "defaultValue")]
	pub DefaultValue:Option<Value>,
	#[serde(alias = "userValue")]
	pub UserValue:Option<Value>,
	#[serde(alias = "userLocalValue")]
	pub UserLocalValue:Option<Value>,
	#[serde(alias = "userRemoteValue")]
	pub UserRemoteValue:Option<Value>,
	#[serde(alias = "workspaceValue")]
	pub WorkspaceValue:Option<Value>,
	#[serde(alias = "workspaceFolderValue")]
	pub WorkspaceFolderValue:Option<Value>,
	#[serde(alias = "memoryValue")]
	pub MemoryValue:Option<Value>,
	#[serde(alias = "policyValue")]
	pub PolicyValue:Option<Value>,
	#[serde(alias = "effectiveValue")]
	pub EffectiveValue:Option<Value>,
	#[serde(alias = "defaultLanguageValue")]
	pub DefaultLanguageValue:Option<Value>,
	#[serde(alias = "userLanguageValue")]
	pub UserLanguageValue:Option<Value>,
	#[serde(alias = "userLocalLanguageValue")]
	pub UserLocalLanguageValue:Option<Value>,
	#[serde(alias = "userRemoteLanguageValue")]
	pub UserRemoteLanguageValue:Option<Value>,
	#[serde(alias = "workspaceLanguageValue")]
	pub WorkspaceLanguageValue:Option<Value>,
	#[serde(alias = "workspaceFolderLanguageValue")]
	pub WorkspaceFolderLanguageValue:Option<Value>,
	#[serde(alias = "memoryLanguageValue")]
	pub MemoryLanguageValue:Option<Value>,
	#[serde(alias = "policyLanguageValue")]
	pub PolicyLanguageValue:Option<Value>,
	#[serde(skip_serializing_if = "Option::is_none")]
	#[serde(alias = "languageIds")]
	pub LanguageIdentifiers:Option<Vec<String>>,
}
