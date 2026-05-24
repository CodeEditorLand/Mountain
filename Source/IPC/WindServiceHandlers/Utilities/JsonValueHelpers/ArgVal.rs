//! `JsonValueHelpers::ArgVal`

use serde_json::Value;

pub fn Fn(Args:&[Value], N:usize) -> Value { Args.get(N).cloned().unwrap_or(Value::Null) }
