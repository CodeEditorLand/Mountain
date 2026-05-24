//! `Params::ObjVal`

use serde_json::{Value, json};

pub fn Fn(P:&Value, Key:&str) -> Value { P.get(Key).cloned().unwrap_or(Value::Null) }
