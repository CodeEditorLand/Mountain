use std::{collections::HashMap, sync::Arc};

use parking_lot::Mutex;

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
pub struct DecorationsState {

	Entries:Arc<Mutex<HashMap<String, Value>>>,
}

impl Default for DecorationsState {

	fn default() -> Self {
		dev_log!("decorations", "[DecorationsState] Initializing default decorations state...");

		Self { Entries:Arc::new(Mutex::new(HashMap::new())) }
	}
}

impl DecorationsState {

	/// Return the JSON decoration value for a URI, or `None` when not set.
	pub fn GetDecoration(&self, Uri:&str) -> Option<Value> { self.Entries.lock().get(Uri).cloned() }

	/// Store or overwrite the decoration for a URI.
	pub fn SetDecoration(&self, Uri:&str, Decoration:Value) {
		let mut Guard = self.Entries.lock();

		Guard.insert(Uri.to_owned(), Decoration);

		dev_log!("decorations", "[DecorationsState] Decoration set for: {}", Uri);
	}

	/// Remove the decoration for a URI.
	pub fn ClearDecoration(&self, Uri:&str) {
		let mut Guard = self.Entries.lock();

		Guard.remove(Uri);

		dev_log!("decorations", "[DecorationsState] Decoration cleared for: {}", Uri);
	}

	/// Return all stored decorations as a cloned map.
	pub fn GetAll(&self) -> HashMap<String, Value> { self.Entries.lock().clone() }
}
