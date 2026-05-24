//! `Params::StringAt`

use serde_json::{Value, json};

pub fn Fn(P:&Value, N:usize) -> String { StrAt(P, N).to_string() }
