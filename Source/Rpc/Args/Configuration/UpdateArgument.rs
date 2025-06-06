// File: Rpc/Args/Configuration/UpdateArgument.rs

use serde::Deserialize;
use serde_json::Value;

// Assuming OverridesDto is accessible from this module's parent
use super::GetConfigurationArgument::OverridesDto;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct UpdateArgument {
	// This u32 should correspond to the CommonConfigurationTarget enum values.
	pub Target:u32,
	pub Key:String,
	pub Value:Value, // Value to set; can be null to remove the key
	pub Overrides:Option<OverridesDto>,
	#[serde(alias = "scopeToLanguage")]
	pub ScopeToLanguage:Option<bool>,
}
