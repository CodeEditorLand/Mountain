

use serde::Deserialize;
use serde_json::Value; // Used for viewColumn which can be a number or an object

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RevealArgument {
	// Identifier of the output channel to reveal (make visible).
	#[serde(alias = "channelId")]
	pub ChannelIdentifier:String,
	// Optional. If true, the focus should not be taken by the revealed output channel.
	// Defaults to false if not provided.
	#[serde(alias = "preserveFocus")]
	pub PreserveFocus:Option<bool>,
	// Optional. Specifies the view column in which to reveal the channel.
	// This can be a number (e.g., ViewColumn enum value) or an object with view column details.
	// The exact structure for an object here depends on VS Code's internal DTO,
	// using `Value` allows flexibility.
	#[serde(alias = "viewColumn")]
	pub ViewColumn:Option<Value>,
}
