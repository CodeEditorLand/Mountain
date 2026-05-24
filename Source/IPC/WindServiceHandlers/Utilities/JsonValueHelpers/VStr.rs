//! Extract a string from a raw JSON string or VS Code `UriComponents` object
//! (`external` / `path` field).

use serde_json::Value;

pub fn Fn(V:&Value) -> Option<String> {
	if let Some(S) = V.as_str() {
		return Some(S.to_string());
	}

	if let Some(Object) = V.as_object() {
		if let Some(S) = Object.get("external").and_then(|V| V.as_str()) {
			return Some(S.to_string());
		}

		if let Some(S) = Object.get("path").and_then(|V| V.as_str()) {
			return Some(S.to_string());
		}
	}

	None
}
