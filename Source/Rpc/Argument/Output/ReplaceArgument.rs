// File: Rpc/Argument/Output/ReplaceArgument.rs

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ReplaceArgument {
	// Identifier of the output channel whose content is to be replaced.
	#[serde(alias = "channelId")]
	pub ChannelIdentifier:String,
	// The new text content that will replace the entire current content of the channel.
	#[serde(alias = "value")]
	pub Content:String,
}
