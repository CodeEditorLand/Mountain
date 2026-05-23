
//! Build a `file://` `UriComponents` from an absolute filesystem path.
//! Path is emitted verbatim - no percent-encoding, no normalisation -
//! mirroring what VS Code's `URI.file(…)` readers expect.

use serde_json::{Value, json};

use crate::IPC::UriComponents::StampMidUri;

pub fn Fn<S:AsRef<str>>(Path:S) -> Value {
	StampMidUri::Fn(json!({
		"scheme": "file",
		"authority": "",
		"path": Path.as_ref(),
		"query": "",
		"fragment": "",
	}))
}
