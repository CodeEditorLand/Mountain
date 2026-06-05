use std::{
	collections::HashSet,
	sync::Arc,
};

use parking_lot::Mutex;

use crate::dev_log;

/// Tracks which URIs have unsaved changes (dirty state).
/// Drives the dirty dot in editor tabs and the explorer badge count.
#[derive(Clone)]
pub struct WorkingCopyState {
	DirtyUris:Arc<Mutex<HashSet<String>>>,
}

impl Default for WorkingCopyState {
	fn default() -> Self {
		dev_log!("workingcopy", "[WorkingCopyState] Initializing default working-copy state...");

		Self { DirtyUris:Arc::new(Mutex::new(HashSet::new())) }
	}
}

impl WorkingCopyState {
	/// Returns `true` if the given URI has unsaved changes.
	pub fn IsDirty(&self, Uri:&str) -> bool {
		self.DirtyUris.lock().contains(Uri)
	}

	/// Mark a URI as dirty or clean.
	pub fn SetDirty(&self, Uri:&str, Dirty:bool) {
		let mut Guard = self.DirtyUris.lock();
		if Dirty {
			Guard.insert(Uri.to_owned());

			dev_log!("workingcopy", "[WorkingCopyState] URI marked dirty: {}", Uri);
		} else {
			Guard.remove(Uri);

			dev_log!("workingcopy", "[WorkingCopyState] URI marked clean: {}", Uri);
		}
	}

	/// Return all URIs with unsaved changes.
	pub fn GetAllDirty(&self) -> Vec<String> {
		self.DirtyUris.lock().iter().cloned().collect()
	}

	/// Return the count of resources with unsaved changes.
	pub fn GetDirtyCount(&self) -> usize { self.DirtyUris.lock().len() }
}