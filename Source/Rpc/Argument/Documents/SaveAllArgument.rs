

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct SaveAllArgument {
	#[serde(alias = "includeUntitled")] // Matches original DTO field name
	pub IncludeUntitled: bool,
}
