//! `NLSResolver::ManifestContainsNLSPlaceholders`

use std::{path::PathBuf, sync::Arc};
use CommonLibrary::{Effect::ApplicationRunTime::ApplicationRunTime as _, FileSystem::ReadFile::ReadFile};
use serde_json::{Map, Value};
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Return `true` if `Value` contains any `%placeholder%` token anywhere in
/// the tree. Used to skip bundle I/O for manifests that have no tokens.
pub fn Fn(Value:&Value) -> bool {
	match Value {
		serde_json::Value::String(Text) => {
			Text.len() >= 2 && Text.starts_with('%') && Text.ends_with('%') && !Text[1..Text.len() - 1].contains('%')
		},

		serde_json::Value::Array(Items) => Items.iter().any(ManifestContainsNLSPlaceholders),

		serde_json::Value::Object(Object) => Object.values().any(ManifestContainsNLSPlaceholders),

		_ => false,
	}
}
