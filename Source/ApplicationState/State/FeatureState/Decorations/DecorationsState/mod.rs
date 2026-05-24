pub mod GetDecoration;
pub mod SetDecoration;
pub mod ClearDecoration;
pub mod GetAll;

use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use serde_json::Value;
use crate::dev_log;

/// A single file/folder decoration: badge letter, tooltip, color hint.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct DecorationData {
	/// Single character badge shown in the explorer (e.g. "M" for modified).
	pub Badge:Option<String>,

	/// Tooltip text displayed on hover.
	pub Tooltip:Option<String>,

	/// Color hint for the item label (theme color ID, e.g.
	/// "gitDecoration.modifiedResourceForeground").
	pub Color:Option<String>,

	/// Whether to propagate the badge to parent folders.
	pub Propagate:Option<bool>,
}

/// Stores per-URI file decorations (git badges, error squiggles, custom
/// badges).
#[derive(Clone)]
pub struct Struct {
	Entries:Arc<StandardMutex<HashMap<String, Value>>>,
}
