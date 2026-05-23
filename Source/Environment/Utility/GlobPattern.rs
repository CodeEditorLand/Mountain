#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Glob pattern extraction helpers for `WorkspaceProvider::FindFiles` calls.
//!
//! VS Code passes glob patterns in several shapes depending on the caller:
//! - Bare string: `"**/*.rs"`
//! - `RelativePattern` object: `{ base, pattern }` or `{ baseUri, pattern }`
//! - Legacy serialised form: `{ value: "**/*.rs" }`
//!
//! These helpers normalise all shapes to a `String` and extract the optional
//! `base` directory for bounded walks.

use serde_json::Value;

/// Extract a glob string from any shape the caller can supply:
/// - Bare string → returned as-is.
/// - Object with `pattern` field (VS Code `RelativePattern`).
/// - Object with `value` field (legacy serialised form).
/// - Object with `Pattern` field (PascalCase variant).
pub(crate) fn ExtractGlobPattern(Pattern:&Value) -> Option<String> {
	if let Some(S) = Pattern.as_str() {
		return Some(S.to_string());
	}

	if let Some(Obj) = Pattern.as_object() {
		if let Some(P) = Obj.get("pattern").and_then(Value::as_str) {
			return Some(P.to_string());
		}

		if let Some(P) = Obj.get("value").and_then(Value::as_str) {
			return Some(P.to_string());
		}

		if let Some(P) = Obj.get("Pattern").and_then(Value::as_str) {
			return Some(P.to_string());
		}
	}

	None
}

/// Extract a `base` directory from a `RelativePattern`-shaped value.
/// VS Code's `RelativePattern` carries `{ base, pattern }` or
/// `{ baseUri, pattern }`. When present, the file walk is restricted to
/// `base`. Returns `None` for plain glob strings.
pub(crate) fn ExtractRelativeBase(Pattern:&Value) -> Option<String> {
	let Obj = Pattern.as_object()?;

	if let Some(B) = Obj.get("base").and_then(Value::as_str) {
		return Some(B.to_string());
	}

	if let Some(B) = Obj.get("baseUri") {
		if let Some(S) = B.as_str() {
			if let Some(Stripped) = S.strip_prefix("file://") {
				return Some(Stripped.to_string());
			}

			return Some(S.to_string());
		}

		if let Some(P) = B.as_object().and_then(|O| O.get("path")).and_then(Value::as_str) {
			return Some(P.to_string());
		}

		if let Some(P) = B.as_object().and_then(|O| O.get("fsPath")).and_then(Value::as_str) {
			return Some(P.to_string());
		}
	}

	None
}
