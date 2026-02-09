//! # Language Detection Utilities
//!
//! Functions for inferring language identifiers from file paths and content.

use std::{ffi::OsStr, path::Path};

/// A simple utility to detect a language identifier string from a file path's
/// extension.
pub fn DetectLanguageIdentifierFromFilePath(Path:&Path) -> String {
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
