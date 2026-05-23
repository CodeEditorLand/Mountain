//! Insert `$mid: 1` into a `UriComponents` object if it isn't already
//! tagged. Non-object values pass through unchanged so call sites can
//! pipe any `serde_json::Value` through without branching first.

use serde_json::{Value, json};

use crate::IPC::UriComponents::MID_URI;

pub fn Fn(Input:Value) -> Value {
	match Input {
		Value::Object(mut Map) => {
			Map.entry("$mid".to_string()).or_insert(json!(MID_URI::VALUE));

			Value::Object(Map)
		},

		Other => Other,
	}
}
