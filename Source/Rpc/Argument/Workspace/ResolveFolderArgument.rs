// File: Rpc/Argument/Workspace/ResolveFolderArgument.rs

use serde::Deserialize;
use serde_json::Value; // For UriComponentsDto which is a generic Value

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ResolveFolderArgument {
	// The UriComponents DTO of a URI that is expected to be within a workspace folder.
	// The handler will attempt to find the workspace folder that contains this URI.
	#[serde(alias = "uriComponentsDto")]
	pub UriComponentsDto:Value,
}
