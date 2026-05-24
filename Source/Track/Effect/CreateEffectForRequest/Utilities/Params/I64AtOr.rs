//! `Params::I64AtOr`

use serde_json::{Value, json};

pub fn Fn(P:&Value, N:usize, Default:i64) -> i64 { P.get(N).and_then(Value::as_i64).unwrap_or(Default) }
