//! `JsonValueHelpers::ArgBoolTrue`

use serde_json::Value;

pub fn Fn(Args:&[Value], N:usize) -> bool { Args.get(N).and_then(Value::as_bool).unwrap_or(true) }
