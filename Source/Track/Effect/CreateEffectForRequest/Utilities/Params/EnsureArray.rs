//! `Params::EnsureArray`

use serde_json::{Value, json};

/// Ensure the value is a JSON array; wraps non-arrays in `[Value]`.
pub fn Fn(P:Value) -> Value { if P.is_array() { P } else { json!([P]) } }
