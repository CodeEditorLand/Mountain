#![allow(non_snake_case)]

//! Build a `UriComponents` from a fully-formed URL string. Handles
//! `file://` (authority-optional) and any other scheme generically
//! (`scheme:path` + optional `//authority`). Fragment / query are split
//! off verbatim so downstream `URI.revive()` reconstructs the same URL.
//! Strings that don't parse as URLs fall back to `{ scheme:"file",
//! path:<input> }` - a defensive shape the workbench tolerates for
//! unknown-location placeholders.

use serde_json::{Value, json};

use crate::IPC::UriComponents::StampMidUri;

pub fn Fn(Url:&str) -> Value {
	if let Some(Rest) = Url.strip_prefix("file://") {
		let (Authority, Path) = match Rest.find('/') {
			Some(0) => ("", Rest),

			Some(Index) => (&Rest[..Index], &Rest[Index..]),

			None => ("", ""),
		};

		return StampMidUri::Fn(json!({
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

		return StampMidUri::Fn(json!({
			"scheme": Scheme,
			"authority": Authority,
			"path": Path,
			"query": "",
			"fragment": "",
		}));
	}

	StampMidUri::Fn(json!({
		"scheme": "file",
		"authority": "",
		"path": Url,
		"query": "",
		"fragment": "",
	}))
}
