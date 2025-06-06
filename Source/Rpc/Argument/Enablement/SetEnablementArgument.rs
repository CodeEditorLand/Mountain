

use serde::Deserialize;
use serde_json::Value;

// This DTO structure assumes `extensionIdDtos` is an array of
// IExtensionIdentifier DTOs and `newState` is a u32 representing the
// `EnablementState` enum.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SetEnablementArgument {
	#[serde(alias = "extensionIdDtos")] // Matches one of the field names used
	pub ExtensionIdentifierDtos: Value, // Array of IExtensionIdentifier DTOs
	#[serde(alias = "newState")] // Matches the field name from track.rs
	pub NewState: u32, // Represents vs_platform_extensions::EnablementState as u32
}
