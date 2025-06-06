// File: Rpc/Argument/Output/RegisterArgument.rs

use serde::Deserialize;
use serde_json::Value; // Used for fileUriDto which can be a UriComponents DTO or null

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct RegisterArgument {
	// The human-readable name for the output channel.
	pub Name:String,
	// Optional. If provided, represents a UriComponents DTO for a file
	// that this output channel might be associated with (e.g., for log channels).
	// Can be null if not file-associated.
	#[serde(alias = "fileUriDto")]
	pub FileUriDto:Option<Value>,
	// Optional. The language identifier to associate with the output channel's content,
	// which can enable syntax highlighting in the output panel.
	#[serde(alias = "languageId")]
	pub LanguageIdentifier:Option<String>,
}
