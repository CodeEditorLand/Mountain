//! `Params::StripFileUri`

use serde_json::{Value, json};

/// Strip a leading `file://` or `file:///` scheme. Handles the
/// `file://localhost/...` form by removing the host segment.
pub fn Fn(Input:&str) -> &str {
	if let Some(Rest) = Input.strip_prefix("file://") {
		if Rest.starts_with('/') {
			return Rest;
		}
		if let Some(Idx) = Rest.find('/') {
			return &Rest[Idx..];
		}
	}
	Input
}
