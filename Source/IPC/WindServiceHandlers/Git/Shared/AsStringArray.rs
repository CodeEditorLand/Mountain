//! Converts a JSON array value to `Vec<String>`.

use serde_json::Value;

pub fn Fn(Value:&Value) -> Vec<String> {
	Value
		.as_array()
		.map(|Arr| Arr.iter().filter_map(|V| V.as_str().map(str::to_string)).collect())
		.unwrap_or_default()
}
