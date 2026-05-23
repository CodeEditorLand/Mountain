//! Serde-Value helpers shared across Wind handlers. `v_str` (`Fn`) extracts a
//! string from either a raw JSON string or a VS Code `UriComponents` object
//! (`external` / `path` field). The `arg_*` family extracts typed scalars
//! from `&[Value]` (Wind handler argument lists) at a given position index.
//! Any new cross-cutting coercer for both shapes belongs here.

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

pub fn arg_str(args:&[Value], n:usize) -> &str { args.get(n).and_then(Value::as_str).unwrap_or("") }

pub fn arg_string(args:&[Value], n:usize) -> String { arg_str(args, n).to_string() }

pub fn arg_string_or(args:&[Value], n:usize, default:&str) -> String {
	args.get(n).and_then(Value::as_str).unwrap_or(default).to_string()
}

pub fn arg_val(args:&[Value], n:usize) -> Value { args.get(n).cloned().unwrap_or(Value::Null) }

pub fn arg_u64(args:&[Value], n:usize) -> u64 { args.get(n).and_then(Value::as_u64).unwrap_or(0) }

pub fn arg_u64_or(args:&[Value], n:usize, default:u64) -> u64 { args.get(n).and_then(Value::as_u64).unwrap_or(default) }

pub fn arg_i64(args:&[Value], n:usize) -> i64 { args.get(n).and_then(Value::as_i64).unwrap_or(0) }

pub fn arg_f64(args:&[Value], n:usize) -> f64 { args.get(n).and_then(Value::as_f64).unwrap_or(0.0) }

pub fn arg_bool(args:&[Value], n:usize) -> bool { args.get(n).and_then(Value::as_bool).unwrap_or(false) }

pub fn arg_bool_true(args:&[Value], n:usize) -> bool { args.get(n).and_then(Value::as_bool).unwrap_or(true) }

pub fn req_str<'a>(args:&'a [Value], n:usize, msg:&str) -> Result<&'a str, String> {
	args.get(n).and_then(Value::as_str).ok_or_else(|| msg.to_string())
}

pub fn req_string(args:&[Value], n:usize, msg:&str) -> Result<String, String> {
	req_str(args, n, msg).map(str::to_string)
}
