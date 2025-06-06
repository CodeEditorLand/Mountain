

use serde::Deserialize;
use serde_json::Value;

// This DTO is used for registering various language feature providers.
// - `Handle`: A u32 handle assigned by Cocoon, which Mountain will store and
//   use to refer to this provider registration, especially for unregistration
//   or when resolving items related to this provider (e.g.,
//   resolveCompletionItem).
// - `SelectorDto`: A DocumentSelector DTO (can be a string, array of strings,
//   or filter object) that specifies which documents this provider applies to.
// - `OptionsDto`: Optional, provider-specific options (e.g., trigger characters
//   for completion). This is a generic `Value` because its structure varies per
//   provider type. The `MainThreadLanguageFeaturesHandler` will parse this into
//   a more specific `SpecificProviderOptionsDto` based on the
//   `ActualProviderType`.
// - `ExtensionIdentifierDto`: An IExtensionIdentifier DTO identifying the
//   extension registering the provider.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RegisterProviderArgument {
	pub Handle:u32,
	#[serde(alias = "selectorDto")]
	pub SelectorDto:Value, // Represents DocumentSelector DTO
	#[serde(alias = "optionsDto")]
	pub OptionsDto:Option<Value>, // Provider-specific options DTO
	#[serde(alias = "extensionIdDto")]
	pub ExtensionIdentifierDto:Value, // Represents IExtensionIdentifier DTO
}
