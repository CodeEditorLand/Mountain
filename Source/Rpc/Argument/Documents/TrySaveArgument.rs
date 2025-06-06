

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TrySaveArgument {
	// Represents UriComponents DTO of the document to save
	#[serde(alias = "uriComponentsDto")]
	pub UriComponentsDto:Value,
}
