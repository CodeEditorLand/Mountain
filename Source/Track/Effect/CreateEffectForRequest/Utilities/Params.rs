//! Positional and named scalar extractors for `Parameters: Value` arguments
//! in `CreateEffectForRequest` domain handlers. `Parameters` is a single
//! `serde_json::Value` that may be a JSON array (positional) or object
//! (named). The `str_obj_or_pos` helper handles the 7 sites that accept both
//! shapes.

use serde_json::{Value, json};

pub fn str_at(p:&Value, n:usize) -> &str { p.get(n).and_then(Value::as_str).unwrap_or("") }

pub fn string_at(p:&Value, n:usize) -> String { str_at(p, n).to_string() }

pub fn string_at_or(p:&Value, n:usize, default:&str) -> String {
	p.get(n).and_then(Value::as_str).unwrap_or(default).to_string()
}

pub fn val_at(p:&Value, n:usize) -> Value { p.get(n).cloned().unwrap_or(Value::Null) }

pub fn u64_at(p:&Value, n:usize) -> u64 { p.get(n).and_then(Value::as_u64).unwrap_or(0) }

pub fn bool_at(p:&Value, n:usize) -> bool { p.get(n).and_then(Value::as_bool).unwrap_or(false) }

pub fn bool_at_true(p:&Value, n:usize) -> bool { p.get(n).and_then(Value::as_bool).unwrap_or(true) }

pub fn i64_at(p:&Value, n:usize) -> i64 { p.get(n).and_then(Value::as_i64).unwrap_or(0) }

pub fn i64_at_or(p:&Value, n:usize, default:i64) -> i64 { p.get(n).and_then(Value::as_i64).unwrap_or(default) }

pub fn u64_at_or(p:&Value, n:usize, default:u64) -> u64 { p.get(n).and_then(Value::as_u64).unwrap_or(default) }

pub fn obj_str<'a>(p:&'a Value, key:&str) -> &'a str { p.get(key).and_then(Value::as_str).unwrap_or("") }

pub fn obj_val(p:&Value, key:&str) -> Value { p.get(key).cloned().unwrap_or(Value::Null) }

pub fn obj_bool(p:&Value, key:&str) -> bool { p.get(key).and_then(Value::as_bool).unwrap_or(false) }

pub fn obj_f64(p:&Value, key:&str) -> Option<f64> { p.get(key).and_then(Value::as_f64) }

pub fn str_obj_or_pos<'a>(p:&'a Value, key:&str, n:usize) -> &'a str {
	if let Some(obj) = p.as_object() {
		obj.get(key).and_then(Value::as_str).unwrap_or("")
	} else {
		p.get(n).and_then(Value::as_str).unwrap_or("")
	}
}

/// Unwrap one level of outer array: `[payload]` → `payload`, else identity.
/// Handles callers that always wrap in an array vs those that send the value
/// directly.
pub fn array_unwrap(p:Value) -> Value { if p.is_array() { p.get(0).cloned().unwrap_or_default() } else { p } }

/// Extract a URI parameter that may arrive as `[uri]`, `{uri:…}`, or bare.
pub fn uri_from_params(p:Value) -> Value {
	if p.is_array() {
		p.get(0).cloned().unwrap_or_default()
	} else {
		p.get("uri").cloned().unwrap_or(p)
	}
}

/// Ensure the value is a JSON array; wraps non-arrays in `[value]`.
pub fn ensure_array(p:Value) -> Value { if p.is_array() { p } else { json!([p]) } }

/// Strip a leading `file://` or `file:///` scheme. Handles the
/// `file://localhost/...` form by removing the host segment.
pub fn strip_file_uri(input:&str) -> &str {
	if let Some(rest) = input.strip_prefix("file://") {
		if rest.starts_with('/') {
			return rest;
		}
		if let Some(idx) = rest.find('/') {
			return &rest[idx..];
		}
	}
	input
}
