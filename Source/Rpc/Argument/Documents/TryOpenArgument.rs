

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct TryOpenArgument {
	// Represents UriComponents DTO or Value::Null for new untitled files
	#[serde(alias = "uriComponentsDto")]
	pub UriComponentsDto:Value,
	#[serde(alias = "languageId")]
	pub LanguageIdentifier:Option<String>,
	pub Content:Option<String>,
}
