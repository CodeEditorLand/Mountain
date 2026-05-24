//! `NLSResolver::LoadNLSBundle`

use std::{path::PathBuf, sync::Arc};
use CommonLibrary::{Effect::ApplicationRunTime::ApplicationRunTime as _, FileSystem::ReadFile::ReadFile};
use serde_json::{Map, Value};
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Load an extension's NLS bundle (`package.nls.json`) into a `{key → string}`
/// map. Returns `None` if absent or unreadable - placeholders remain as-is.
/// Entries can be bare strings or `{message, comment}` objects; only `message`
/// is kept. The `PlaceholdersNeeded` flag downgrades the "no bundle" warning
/// when the manifest has no `%placeholder%` entries (absence is benign).
pub async fn Fn(
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
			obj.get("message").and_then(|m| m.as_str()).map(|S| s.to_string())
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
