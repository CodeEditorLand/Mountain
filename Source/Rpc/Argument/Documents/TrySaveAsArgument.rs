

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TrySaveAsArgument {
	// Represents UriComponents DTO of the original document
	#[serde(alias = "originalUriComponentsDto")]
	pub OriginalUriComponentsDto:Value,
	// Represents optional UriComponents DTO for the new save location/name
	#[serde(alias = "newTargetUriComponentsDto")]
	pub NewTargetUriComponentsDto:Option<Value>,
}
