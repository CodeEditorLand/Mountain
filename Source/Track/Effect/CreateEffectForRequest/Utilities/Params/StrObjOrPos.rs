//! `Params::StrObjOrPos`

use serde_json::{Value, json};

pub fn Fn<'a>(P:&'a Value, Key:&str, N:usize) -> &'a str {
	if let Some(Obj) = P.as_object() {
		Obj.get(Key).and_then(Value::as_str).unwrap_or("")
	} else {
		P.get(N).and_then(Value::as_str).unwrap_or("")
	}
}
