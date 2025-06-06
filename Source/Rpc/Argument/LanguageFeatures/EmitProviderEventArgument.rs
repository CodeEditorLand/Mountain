

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct EmitProviderEventArgument {
	// The handle that was returned when the provider, which might emit this event, was registered.
	// This handle is used by the main thread to know which provider's event listener to trigger.
	#[serde(alias = "eventHandle")]
	pub EventHandle:u32,
	// Optional arguments for the event, specific to the type of provider event
	// (e.g., for onDidChangeCodeLenses, this might be undefined or null).
	#[serde(alias = "eventArgument")]
	pub EventArgument:Option<Value>,
}
