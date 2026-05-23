#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Serde-Value helpers shared across Wind handlers. `v_str` extracts a
//! string from either a raw JSON string or a VS Code `UriComponents`
//! object (`external` / `path` field). Any new cross-cutting coercer that
//! accepts both shapes belongs here.

use serde_json::Value;

pub fn Fn(Value:&Value) -> Option<String> {
	if let Some(s) = Value.as_str() {
		return Some(s.to_string());
	}

	if let Some(Object) = Value.as_object() {
		if let Some(s) = Object.get("external").and_then(|V| V.as_str()) {
			return Some(s.to_string());
		}

		if let Some(s) = Object.get("path").and_then(|V| V.as_str()) {
			return Some(s.to_string());
		}
	}

	None
}
