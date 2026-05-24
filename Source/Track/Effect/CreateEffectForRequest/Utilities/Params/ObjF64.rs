//! `Params::ObjF64`

use serde_json::{Value, json};

pub fn Fn(P:&Value, Key:&str) -> Option<f64> { P.get(Key).and_then(Value::as_f64) }
