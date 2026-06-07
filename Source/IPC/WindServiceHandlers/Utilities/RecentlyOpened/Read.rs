//! Reads and parses RecentlyOpened.json. Degrades to empty envelope on error.

use serde_json::{Value, json};

pub fn Fn() -> Result<Value, String> {

	let Path = super::Path::Fn();

	match std::fs::read_to_string(&Path) {
		Ok(Contents) => {
			match serde_json::from_str::<Value>(&Contents) {
				Ok(Parsed) => Ok(Parsed),

				Err(_) => Ok(json!({ "workspaces": [], "files": [] })),
			}
		},

		Err(_) => Ok(json!({ "workspaces": [], "files": [] })),
	}
}
