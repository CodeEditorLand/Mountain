//! `Params::StringAtOr`

use serde_json::{Value, json};

pub fn Fn(P:&Value, N:usize, Default:&str) -> String { P.get(N).and_then(Value::as_str).unwrap_or(Default).to_string() }
