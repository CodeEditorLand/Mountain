//! `JsonValueHelpers::ArgStringOr`

use serde_json::Value;

pub fn Fn(Args:&[Value], N:usize, Default:&str) -> String {
	Args.get(N).and_then(Value::as_str).unwrap_or(Default).to_string()
}
