//! `Params::StrAt`

use serde_json::{Value, json};

pub fn Fn<'a>(P:&'a Value, N:usize) -> &'a str { P.get(N).and_then(Value::as_str).unwrap_or("") }
