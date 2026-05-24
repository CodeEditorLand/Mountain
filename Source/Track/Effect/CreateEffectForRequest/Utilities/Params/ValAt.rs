//! `Params::ValAt`

use serde_json::{Value, json};

pub fn Fn(P:&Value, N:usize) -> Value { P.get(N).cloned().unwrap_or(Value::Null) }
