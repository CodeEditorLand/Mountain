#![allow(non_snake_case, dead_code)]

//! Shared helpers for emitting VS Code `UriComponents` payloads.
//!
//! VS Code's IPC reviver (`src/vs/base/common/uriIpc.ts`
//! `_transformIncomingURIs`) walks every response object and only calls
//! `URI.revive()` on nested objects tagged with the marshalling marker
//! `$mid === MarshalledId.Uri` (= 1). An untagged `UriComponents` reaches
//! callers as a plain bag - `uri.with is not a function`, `uri.fsPath`
//! undefined - and the sidebar / icon loader / joinPath chain silently
//! breaks.
//!
//! Every handler that returns a URI-shaped payload to the renderer has to
//! stamp the marker. Keep construction centralized here so we can't lose a
//! call site to inline `json!` builders.

use serde_json::{Value, json};

/// `MarshalledId.Uri` from VS Code's `src/vs/base/common/marshallingIds.ts`.
/// The reviver keys off this exact value; changing VS Code's enum would
/// require updating here in lockstep (search for `MID_URI` callers).
pub const MID_URI:u64 = 1;

/// Insert `$mid: 1` into a `UriComponents` object if it isn't already tagged.
/// Non-object values pass through unchanged - that lets call sites pipe any
/// `serde_json::Value` they already have through the helper without branching
/// on the variant first.
pub fn StampMidUri(Input:Value) -> Value {
	match Input {
		Value::Object(mut Map) => {
			Map.entry("$mid".to_string()).or_insert(json!(MID_URI));
			Value::Object(Map)
		},
		Other => Other,
	}
}

/// Build a `file://` `UriComponents` from an absolute filesystem path
/// string. Path is emitted verbatim - no percent-encoding, no normalisation -
/// which mirrors what VS Code's own `URI.file(…)` / `URI.parse(…)` path
/// readers expect (they undo any encoding themselves when reading `.fsPath`).
pub fn FromFilePath<S:AsRef<str>>(Path:S) -> Value {
	StampMidUri(json!({
		"scheme": "file",
		"authority": "",
		"path": Path.as_ref(),
		"query": "",
		"fragment": "",
	}))
}

/// Build a `UriComponents` from a fully-formed URL string. Handles `file://`
/// (authority-optional) and any other scheme generically (`scheme:path` +
/// optional `//authority`). Fragment / query are split off verbatim so
/// downstream `URI.revive()` reconstructs the same URL. Strings that don't
/// parse as URLs fall back to `{ scheme:"file", path:<input> }` - a defensive
/// shape the workbench still tolerates for "unknown location" placeholders.
pub fn FromUrl(Url:&str) -> Value {
	if let Some(Rest) = Url.strip_prefix("file://") {
		let (Authority, Path) = match Rest.find('/') {
			Some(0) => ("", Rest),
			Some(Index) => (&Rest[..Index], &Rest[Index..]),
			None => ("", ""),
		};
		return StampMidUri(json!({
			"scheme": "file",
			"authority": Authority,
			"path": Path,
			"query": "",
			"fragment": "",
		}));
	}
	if let Some((Scheme, PathPart)) = Url.split_once(':') {
		let Trimmed = PathPart.trim_start_matches("//");
		let (Authority, Path) = match Trimmed.find('/') {
			Some(0) => ("", Trimmed),
			Some(Index) => (&Trimmed[..Index], &Trimmed[Index..]),
			None => ("", Trimmed),
		};
		return StampMidUri(json!({
			"scheme": Scheme,
			"authority": Authority,
			"path": Path,
			"query": "",
			"fragment": "",
		}));
	}
	StampMidUri(json!({
		"scheme": "file",
		"authority": "",
		"path": Url,
		"query": "",
		"fragment": "",
	}))
}

/// Normalise an `extensionLocation` (or any similar) field that arrives as
/// either a URL string, a pre-built UriComponents object (possibly
/// already tagged), or is missing / null. The output is always an object
/// with `$mid: 1` and the five URI fields.
pub fn Normalize(Raw:Option<&Value>) -> Value {
	match Raw {
		Some(Value::Object(Map)) if Map.contains_key("scheme") => StampMidUri(Value::Object(Map.clone())),
		Some(Value::String(Url)) => FromUrl(Url),
		_ => FromFilePath("/extensions/unknown"),
	}
}
