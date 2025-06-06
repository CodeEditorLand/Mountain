
// Defines the data structure for storing information about a single
// registered language feature provider.

#![allow(non_snake_case, non_camel_case_types)]

use Common::LanguageFeatureEffect::{OptionsDto::SpecificProviderOptionsDto, ProviderType as LanguageProviderType};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Represents the registration details for a language feature provider.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ProviderRegistration {
	// The handle assigned by Cocoon to this provider registration.
	pub Handle:u32,
	// The type of feature this provider implements (e.g., Hover, Completion).
	pub ProviderType:LanguageProviderType,
	// The DocumentSelector DTO that determines which documents this provider applies to.
	pub Selector:Value,
	// The identifier of the sidecar process where this provider is running.
	pub SidecarIdentifier:String,
	// The IExtensionIdentifier DTO of the extension that registered this provider.
	pub ExtensionIdentifier:Value,
	// Optional, provider-specific configuration options.
	#[serde(skip_serializing_if = "Option::is_none")]
	pub Options:Option<SpecificProviderOptionsDto>,
}
