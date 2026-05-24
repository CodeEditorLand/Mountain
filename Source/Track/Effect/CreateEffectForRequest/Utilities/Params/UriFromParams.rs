//! `Params::UriFromParams`

use serde_json::{Value, json};

/// Extract a URI parameter that may arrive as `[Uri]`, `{uri:…}`, or bare.
pub fn Fn(P:Value) -> Value {
	if P.is_array() {
		P.get(0).cloned().unwrap_or_default()
	} else {
		P.get("uri").cloned().unwrap_or(P)
	}
}
