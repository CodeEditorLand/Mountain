//! `Params::BoolAtTrue`

use serde_json::{Value, json};

pub fn Fn(P:&Value, N:usize) -> bool { P.get(N).and_then(Value::as_bool).unwrap_or(true) }
