

use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ShowArgument {
	// The unique numeric identifier of the terminal instance to show.
	pub Id:u64,
	// Optional. If true, the terminal should be shown without taking focus.
	// Defaults to false if not provided (terminal takes focus).
	#[serde(alias = "preserveFocus")]
	pub PreserveFocus:Option<bool>,
}
