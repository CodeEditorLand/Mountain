//! Expand a leading `~/` against `$HOME`. Returns the input unchanged if
//! `HOME` is unset or the path doesn't start with `~/`.

use std::path::PathBuf;

/// Public entry point for this module.
pub fn Fn(Raw:&str) -> PathBuf {
	if let Some(Stripped) = Raw.strip_prefix("~/") {
		if let Ok(Home) = std::env::var("HOME") {
			return PathBuf::from(Home).join(Stripped);
		}
	}

	PathBuf::from(Raw)
}
