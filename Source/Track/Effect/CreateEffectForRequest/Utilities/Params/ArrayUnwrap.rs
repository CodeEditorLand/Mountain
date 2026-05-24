//! `Params::ArrayUnwrap`

use serde_json::{Value, json};

/// Unwrap one level of outer array: `[Payload]` → `Payload`, else identity.
pub fn Fn(P:Value) -> Value { if P.is_array() { P.get(0).cloned().unwrap_or_default() } else { P } }
