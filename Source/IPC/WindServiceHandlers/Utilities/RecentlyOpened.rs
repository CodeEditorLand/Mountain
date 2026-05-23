
//! Recently-opened workspaces/files persistence.
//! File lives at `~/.fiddee/workspaces/RecentlyOpened.json` (resolved
//! through the `FiddeeRoot` atom). Parse failures degrade to an empty
//! `{workspaces, files}` envelope so the UI never sees a missing field.

use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Utilities::FiddeeRoot::FiddeeRoot;

pub fn RecentlyOpenedPath() -> std::path::PathBuf { FiddeeRoot().join("workspaces").join("RecentlyOpened.json") }

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
