#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! NLS (National Language Support) placeholder resolution for extension
//! manifests. VS Code extensions embed `%key%` tokens in their `package.json`
//! that are resolved at runtime from a `package.nls.json` bundle.
//!
//! Three functions work together:
//! - `ManifestContainsNLSPlaceholders` - fast pre-scan to skip bundle I/O
//! - `LoadNLSBundle` - read and parse `package.nls.json`
//! - `ResolveNLSPlaceholdersInner` - in-place recursive token substitution

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{Effect::ApplicationRunTime::ApplicationRunTime as _, FileSystem::ReadFile::ReadFile};
use serde_json::{Map, Value};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Return `true` if `Value` contains any `%placeholder%` token anywhere in
/// the tree. Used to skip bundle I/O for manifests that have no tokens.
pub fn ManifestContainsNLSPlaceholders(Value:&Value) -> bool {
	match Value {
		serde_json::Value::String(Text) => {
			Text.len() >= 2
				&& Text.starts_with('%')
				&& Text.ends_with('%')
				&& !Text[1..Text.len() - 1].contains('%')
		},
		serde_json::Value::Array(Items) => Items.iter().any(ManifestContainsNLSPlaceholders),
		serde_json::Value::Object(Object) => Object.values().any(ManifestContainsNLSPlaceholders),
		_ => false,
	}
}

/// Load an extension's NLS bundle (`package.nls.json`) into a `{key → string}`
/// map. Returns `None` if absent or unreadable - placeholders remain as-is.
/// Entries can be bare strings or `{message, comment}` objects; only `message`
/// is kept. The `PlaceholdersNeeded` flag downgrades the "no bundle" warning
/// when the manifest has no `%placeholder%` entries (absence is benign).
pub async fn LoadNLSBundle(
	RunTime:&Arc<ApplicationRunTime>,
	ExtensionPath:&PathBuf,
	PlaceholdersNeeded:bool,
) -> Option<Map<String, Value>> {
	let NLSPath = ExtensionPath.join("package.nls.json");
	let Content = match RunTime.Run(ReadFile(NLSPath.clone())).await {
		Ok(Bytes) => Bytes,
		Err(Error) => {
			if PlaceholdersNeeded {
				dev_log!("nls", "[LandFix:NLS] no bundle for {} ({})", ExtensionPath.display(), Error);
			} else {
				dev_log!(
					"nls",
					"[LandFix:NLS] {} has no placeholders, no bundle needed",
					ExtensionPath.display()
				);
			}
			return None;
		},
	};
	let Parsed:Value = match serde_json::from_slice(&Content) {
		Ok(V) => V,
		Err(Error) => {
			dev_log!("nls", "warn: [LandFix:NLS] failed to parse {}: {}", NLSPath.display(), Error);
			return None;
		},
	};
	let Object = Parsed.as_object()?;
	let mut Resolved = Map::with_capacity(Object.len());
	for (Key, RawValue) in Object {
		let Text = if let Some(s) = RawValue.as_str() {
			Some(s.to_string())
		} else if let Some(obj) = RawValue.as_object() {
			obj.get("message").and_then(|m| m.as_str()).map(|s| s.to_string())
		} else {
			None
		};
		if let Some(t) = Text {
			Resolved.insert(Key.clone(), Value::String(t));
		}
	}
	dev_log!(
		"nls",
		"[LandFix:NLS] loaded {} keys for {}",
		Resolved.len(),
		ExtensionPath.display()
	);
	Some(Resolved)
}

/// In-place recursive substitution of `%key%` tokens using the NLS map.
/// `Replaced` and `Unresolved` accumulate counts for the outer scanner's
/// one-line summary log.
pub fn ResolveNLSPlaceholdersInner(
	Value:&mut Value,
	NLS:&Map<String, Value>,
	Replaced:&mut u32,
	Unresolved:&mut u32,
) {
	match Value {
		serde_json::Value::String(Text) => {
			if Text.len() >= 2 && Text.starts_with('%') && Text.ends_with('%') {
				let Key = &Text[1..Text.len() - 1];
				if !Key.is_empty() && !Key.contains('%') {
					if let Some(Replacement) = NLS.get(Key).and_then(|v| v.as_str()) {
						*Text = Replacement.to_string();
						*Replaced += 1;
					} else {
						*Unresolved += 1;
					}
				}
			}
		},
		serde_json::Value::Array(Items) => {
			for Item in Items {
				ResolveNLSPlaceholdersInner(Item, NLS, Replaced, Unresolved);
			}
		},
		serde_json::Value::Object(Map) => {
			for (_, FieldValue) in Map {
				ResolveNLSPlaceholdersInner(FieldValue, NLS, Replaced, Unresolved);
			}
		},
		_ => {},
	}
}
