//! # Language Detection Utilities
//!
//! Functions for inferring language identifiers from file paths and content.

use std::{ffi::OsStr, path::Path};

/// Infers a language identifier string from the extension of a file path.
pub fn Fn(Path:&Path) -> String {
	match Path.extension().and_then(OsStr::to_str) {
		Some("js") | Some("mjs") | Some("cjs") => "javascript",

		Some("ts") | Some("mts") | Some("cts") => "typescript",

		Some("jsx") => "javascriptreact",

		Some("tsx") => "typescriptreact",

		Some("rs") => "rust",

		Some("md") => "markdown",

		Some("json") => "json",

		Some("html") => "html",

		Some("css") => "css",

		_ => "plaintext",
	}
	.to_string()
}
