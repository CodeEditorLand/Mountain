//! Extract a filesystem path from the first arg of `extensions:install`.
//! Wind's install dialog hands us either a raw string ("file:///..." or
//! absolute path) or a Tauri-serialised `UriComponents` object; both forms
//! collapse to a single `PathBuf` here. `None` when the arg is missing,
//! malformed, or carries a non-file scheme.

use std::path::PathBuf;

use serde_json::Value;

pub fn Fn(Args:&[Value]) -> Option<PathBuf> {
	let Raw = Args.first()?;

	let RawString = if let Some(AsStr) = Raw.as_str() {
		AsStr.to_string()
	} else if let Some(AsObject) = Raw.as_object() {
		// Wind can pass a UriComponents object; pull the conventional fields.
		if let Some(External) = AsObject.get("external").and_then(|V| V.as_str()) {
			External.to_string()
		} else if let Some(Path) = AsObject.get("path").and_then(|V| V.as_str()) {
			Path.to_string()
		} else {
			return None;
		}
	} else {
		return None;
	};

	if let Ok(Parsed) = url::Url::parse(&RawString) {
		if Parsed.scheme() == "file" {
			return Some(Parsed.to_file_path().unwrap_or_else(|_| PathBuf::from(Parsed.path())));
		}
	}

	Some(PathBuf::from(RawString))
}
