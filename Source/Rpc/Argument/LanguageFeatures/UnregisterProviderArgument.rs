// File: Rpc/Argument/LanguageFeatures/UnregisterProviderArgument.rs

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct UnregisterProviderArgument {
	// The handle that was returned by Cocoon when the provider was initially registered.
	// This handle is used by Mountain to identify which provider registration to remove.
	pub Handle:u32,
}
