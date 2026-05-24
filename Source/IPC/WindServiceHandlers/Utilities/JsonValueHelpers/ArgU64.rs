//! `JsonValueHelpers::ArgU64`

use serde_json::Value;

pub fn Fn(Args:&[Value], N:usize) -> u64 { Args.get(N).and_then(Value::as_u64).unwrap_or(0) }
