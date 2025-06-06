

use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct ExecuteCommandArgument {
	// Renamed from `command_id` to match original DTO structure from track.rs if it was `id`
	#[serde(alias = "id")]
	pub CommandIdentifier:String,
	// Renamed from `command_args` to match original DTO structure from track.rs if it was `args`
	#[serde(alias = "args")]
	pub CommandArgument:Vec<Value>,
}
