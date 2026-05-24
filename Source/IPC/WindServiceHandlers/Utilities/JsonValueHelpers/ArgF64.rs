//! `JsonValueHelpers::ArgF64`

use serde_json::Value;

pub fn Fn(Args:&[Value], N:usize) -> f64 { Args.get(N).and_then(Value::as_f64).unwrap_or(0.0) }
