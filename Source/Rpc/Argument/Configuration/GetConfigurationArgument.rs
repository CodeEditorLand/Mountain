// File: Rpc/Argument/Configuration/GetConfigurationArgument.rs

use serde::Deserialize;
use serde_json::Value;

// This DTO is also used by UpdateArgument and InspectArgument.
#[derive(Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "PascalCase")]
pub struct OverridesDto {
	pub Resource:Option<Value>, // Represents a UriComponents DTO
	#[serde(alias = "overrideIdentifier")]
	pub OverrideIdentifier:Option<String>, // Typically a language ID
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct GetConfigurationArgument {
	pub Section:Option<String>,
	pub Overrides:Option<OverridesDto>,
	#[serde(alias = "scopeToLanguage")]
	pub ScopeToLanguage:Option<bool>,
}
