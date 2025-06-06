// File: Rpc/Argument/Configuration/InspectArgument.rs

use serde::Deserialize;

// Assuming OverridesDto is already defined and accessible from this module's parent
// 
use super::GetConfigurationArgument::OverridesDto;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct InspectArgument {
	pub Key:String,
	pub Overrides:Option<OverridesDto>,
}
