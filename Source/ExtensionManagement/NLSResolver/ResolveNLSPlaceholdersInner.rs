//! `NLSResolver::ResolveNLSPlaceholdersInner`

use std::{path::PathBuf, sync::Arc};
use CommonLibrary::{Effect::ApplicationRunTime::ApplicationRunTime as _, FileSystem::ReadFile::ReadFile};
use serde_json::{Map, Value};
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// In-place recursive substitution of `%key%` tokens using the NLS map.
/// `Replaced` and `Unresolved` accumulate counts for the outer scanner's
/// one-line summary log.
pub fn Fn(Value:&mut Value, NLS:&Map<String, Value>, Replaced:&mut u32, Unresolved:&mut u32) {
	match Value {
		serde_json::Value::String(Text) => {
			if Text.len() >= 2 && Text.starts_with('%') && Text.ends_with('%') {
				let Key = &Text[1..Text.len() - 1];

				if !Key.is_empty() && !Key.contains('%') {
					if let Some(Replacement) = NLS.get(Key).and_then(|V| v.as_str()) {
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
