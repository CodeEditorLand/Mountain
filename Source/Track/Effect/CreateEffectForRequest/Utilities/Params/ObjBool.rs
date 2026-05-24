//! `Params::ObjBool`

use serde_json::{Value, json};

pub fn Fn(P:&Value, Key:&str) -> bool { P.get(Key).and_then(Value::as_bool).unwrap_or(false) }
