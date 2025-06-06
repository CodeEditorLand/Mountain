// File: Rpc/Argument/Window/OpenUriArgument.rs

use serde::Deserialize;
use serde_json::Value; // For UriComponentsDto and potentially complex options

// Defines options for the OpenUri operation.
#[derive(Deserialize, Debug, Clone, Default)] // Added Default
#[serde(rename_all = "PascalCase")]
pub struct OptionsDto {
	// Renamed from OpenUriOption to OptionsDto for consistency
	// If true, allows opening URIs with schemes not natively handled by the OS
	// (e.g., custom extension schemes), provided an opener is registered.
	#[serde(alias = "allowExternalSchemes")]
	pub AllowExternalSchemes:Option<bool>,
	// If true, allows openers contributed by extensions to handle the URI.
	// Can also be a string specifying a particular opener ID.
	#[serde(alias = "allowContributedOpeners")]
	pub AllowContributedOpeners:Option<Value>, // Can be bool or string
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct OpenUriArgument {
	// The UriComponents DTO of the URI to be opened.
	#[serde(alias = "uriComponentsDto")]
	pub UriComponentsDto:Value,
	// Optional settings for the open operation.
	pub Options:Option<OptionsDto>,
}
