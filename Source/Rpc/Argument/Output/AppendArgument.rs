// File: Rpc/Argument/Output/AppendArgument.rs

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct AppendArgument {
	// Identifier of the output channel to append to.
	#[serde(alias = "channelId")]
	pub ChannelIdentifier:String,
	// The text content to append to the channel.
	#[serde(alias = "value")]
	pub Content:String,
}
