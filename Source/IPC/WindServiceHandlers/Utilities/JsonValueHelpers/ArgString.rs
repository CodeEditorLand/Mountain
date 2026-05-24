//! `JsonValueHelpers::ArgString`

use serde_json::Value;

pub fn Fn(Args:&[Value], N:usize) -> String { ArgStr(Args, N).to_string() }
