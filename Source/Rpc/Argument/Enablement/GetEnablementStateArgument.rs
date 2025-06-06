

use serde::Deserialize;
use serde_json::Value;

// This DTO structure assumes `extensionIdDto` is an IExtensionIdentifier DTO
// as defined in VS Code's extension host protocol (typically an object with
// `value` and optionally `uuid`).
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct GetEnablementStateArgument {
	#[serde(alias = "extensionIdDto")] // Matches the field name from track.rs
	pub ExtensionIdentifierDto: Value, // Represents IExtensionIdentifier DTO
}
