//! `Params::I64At`

use serde_json::{Value, json};

pub fn Fn(P:&Value, N:usize) -> i64 { P.get(N).and_then(Value::as_i64).unwrap_or(0) }
