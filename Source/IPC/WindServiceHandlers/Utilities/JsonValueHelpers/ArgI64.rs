//! `JsonValueHelpers::ArgI64`

use serde_json::Value;

pub fn Fn(Args:&[Value], N:usize) -> i64 { Args.get(N).and_then(Value::as_i64).unwrap_or(0) }
