// File: Rpc/Argument/Output/IdArgument.rs

use serde::Deserialize;

// This DTO is used for RPC calls that only require an output channel's
// identifier, such as Clear, Close, and Dispose operations.
#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct IdArgument {
	// Identifier of the target output channel.
	#[serde(alias = "channelId")]
	pub ChannelIdentifier:String,
}
