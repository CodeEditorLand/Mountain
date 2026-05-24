//! `Params::U64AtOr`

use serde_json::{Value, json};

pub fn Fn(P:&Value, N:usize, Default:u64) -> u64 { P.get(N).and_then(Value::as_u64).unwrap_or(Default) }
