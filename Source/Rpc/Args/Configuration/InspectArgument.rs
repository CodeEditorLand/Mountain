// File: Rpc/Args/Configuration/InspectArgument.rs

use serde::Deserialize;

// Assuming OverridesDto is already defined and accessible from this module's parent
// (e.g., re-exported from Rpc/Args/Configuration/Mod.rs or GetConfigurationArgument.rs)
use super::GetConfigurationArgument::OverridesDto;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct InspectArgument {
	pub Key:String,
	pub Overrides:Option<OverridesDto>,
}
