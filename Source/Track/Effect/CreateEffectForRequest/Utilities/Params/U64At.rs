//! `Params::U64At`

use serde_json::{Value, json};

pub fn Fn(P:&Value, N:usize) -> u64 { P.get(N).and_then(Value::as_u64).unwrap_or(0) }
