

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)] // Added Clone
#[serde(rename_all = "PascalCase")]
pub struct RegisterArgument {
	// Assuming `id` was the field name in the original JSON DTO from track.rs
	#[serde(alias = "id")]
	pub Identifier:String,
}
