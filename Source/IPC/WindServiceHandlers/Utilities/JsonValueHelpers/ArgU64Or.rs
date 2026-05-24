//! `JsonValueHelpers::ArgU64Or`

use serde_json::Value;

pub fn Fn(Args:&[Value], N:usize, Default:u64) -> u64 { Args.get(N).and_then(Value::as_u64).unwrap_or(Default) }
