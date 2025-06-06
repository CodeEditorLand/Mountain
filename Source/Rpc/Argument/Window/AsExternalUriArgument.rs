// File: Rpc/Argument/Window/AsExternalUriArgument.rs

use serde::Deserialize;
use serde_json::Value; // For UriComponentsDto which is a generic Value

// Defines options for the AsExternalUri operation.
#[derive(Deserialize, Debug, Clone, Default)] // Added Default
#[serde(rename_all = "PascalCase")]
pub struct OptionsDto {
	// Renamed from AsExternalUriOption to OptionsDto for consistency
	// If true, allows openers contributed by extensions to handle the URI.
	#[serde(alias = "allowContributedOpeners")]
	pub AllowContributedOpeners:Option<bool>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AsExternalUriArgument {
	// The UriComponents DTO of the URI to be converted to its external form.
	#[serde(alias = "uriComponentsDto")]
	pub UriComponentsDto:Value,
	// Optional settings for the operation.
	pub Options:Option<OptionsDto>,
}
