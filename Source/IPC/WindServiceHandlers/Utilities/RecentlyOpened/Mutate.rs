//! Reads, mutates, and writes back RecentlyOpened.json atomically.

use serde_json::{Value, json};

pub fn Fn<F:FnOnce(&mut serde_json::Map<String, Value>)>(Apply:F) {
	let Path = super::Path::Fn();

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
