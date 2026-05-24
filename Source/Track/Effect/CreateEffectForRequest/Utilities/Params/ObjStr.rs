//! `Params::ObjStr`

use serde_json::{Value, json};

pub fn Fn<'a>(P:&'a Value, Key:&str) -> &'a str { P.get(Key).and_then(Value::as_str).unwrap_or("") }
