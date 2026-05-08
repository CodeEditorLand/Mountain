#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Recently-opened workspaces/files persistence.
//! File lives at `~/.land/workspaces/RecentlyOpened.json`. Parse failures
//! degrade to an empty `{workspaces, files}` envelope so the UI never
//! sees a missing field.

use serde_json::{Value, json};

pub fn RecentlyOpenedPath() -> std::path::PathBuf {
	let Home = std::env::var("HOME")
		.or_else(|_| std::env::var("USERPROFILE"))
		.unwrap_or_default();

	std::path::PathBuf::from(Home)
		.join(".land")
		.join("workspaces")
		.join("RecentlyOpened.json")
}

pub fn ReadRecentlyOpened() -> Result<Value, String> {
	let Path = RecentlyOpenedPath();

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

pub fn MutateRecentlyOpened<F:FnOnce(&mut serde_json::Map<String, Value>)>(Apply:F) {
	let Path = RecentlyOpenedPath();

	let mut Parsed:serde_json::Map<String, Value> = std::fs::read_to_string(&Path)
		.ok()
		.and_then(|Contents| serde_json::from_str::<Value>(&Contents).ok())
		.and_then(|V| V.as_object().cloned())
		.unwrap_or_default();

	if !Parsed.contains_key("workspaces") {
		Parsed.insert("workspaces".into(), json!([]));
	}

	if !Parsed.contains_key("files") {
		Parsed.insert("files".into(), json!([]));
	}

	Apply(&mut Parsed);

	if let Some(Parent) = Path.parent() {
		let _ = std::fs::create_dir_all(Parent);
	}

	if let Ok(Serialised) = serde_json::to_vec_pretty(&Value::Object(Parsed)) {
		let _ = std::fs::write(&Path, Serialised);
	}
}
