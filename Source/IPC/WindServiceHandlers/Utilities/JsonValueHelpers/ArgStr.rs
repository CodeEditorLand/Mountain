//! `JsonValueHelpers::ArgStr`

use serde_json::Value;

pub fn Fn(Args:&[Value], N:usize) -> &str { Args.get(N).and_then(Value::as_str).unwrap_or("") }
