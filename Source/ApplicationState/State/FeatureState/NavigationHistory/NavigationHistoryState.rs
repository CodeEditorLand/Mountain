use std::sync::Arc;

use parking_lot::Mutex;

use crate::dev_log;

/// Tracks the editor navigation history stack (back/forward).
///
/// Implements a cursor-based navigation stack where `Index` points to the
/// current position. GoBack decrements the index; GoForward increments it.
/// Pushing a new entry truncates forward history.
#[derive(Clone)]
pub struct NavigationHistoryState {
	/// The ordered list of visited URIs (oldest first).
	Stack:Arc<Mutex<Vec<String>>>,

	/// Current position in the stack (0-based). Points to the active entry.
	Index:Arc<Mutex<usize>>,
}

impl Default for NavigationHistoryState {
	fn default() -> Self {
		dev_log!(
			"history",
			"[NavigationHistoryState] Initializing default navigation history state..."
		);

		Self {
			Stack:Arc::new(Mutex::new(Vec::new())),

			Index:Arc::new(Mutex::new(0)),
		}
	}
}

impl NavigationHistoryState {
	/// Returns `true` if there is a previous location to navigate to.
	pub fn CanGoBack(&self) -> bool {
		let Stack = self.Stack.lock().len();

		let Index = *self.Index.lock();

		Stack > 0 && Index > 0
	}

	/// Returns `true` if there is a next location to navigate to.
	pub fn CanGoForward(&self) -> bool {
		let Stack = self.Stack.lock().len();

		let Index = *self.Index.lock();

		Stack > 0 && Index + 1 < Stack
	}

	/// Decrement the history index (go back). Returns the URI now active, or
	/// `None` if already at the beginning.
	pub fn GoBack(&self) -> Option<String> {
		let Stack = self.Stack.lock();

		if Stack.is_empty() {
			return None;
		}

		let mut Index = self.Index.lock();

		if *Index == 0 {
			return None;
		}

		*Index -= 1;
		let Uri = Stack.get(*Index).cloned();

		dev_log!("history", "[NavigationHistoryState] GoBack → index={} uri={:?}", *Index, Uri);

		Uri
	}

	/// Increment the history index (go forward). Returns the URI now active,
	/// or `None` if already at the end.
	pub fn GoForward(&self) -> Option<String> {
		let Stack = self.Stack.lock();

		if Stack.is_empty() {
			return None;
		}

		let mut Index = self.Index.lock();

		if *Index + 1 >= Stack.len() {
			return None;
		}

		*Index += 1;
		let Uri = Stack.get(*Index).cloned();

		dev_log!("history", "[NavigationHistoryState] GoForward → index={} uri={:?}", *Index, Uri);

		Uri
	}

	/// Push a URI onto the navigation stack. Truncates any forward history
	/// beyond the current index.
	pub fn Push(&self, Uri:String) {
		let mut Stack = self.Stack.lock();
		let mut Index = self.Index.lock();

		// Truncate forward history
		let NewIndex = if Stack.is_empty() { 0 } else { *Index + 1 };

		Stack.truncate(NewIndex);

		Stack.push(Uri.clone());

		*Index = Stack.len() - 1;
		dev_log!("history", "[NavigationHistoryState] Push uri={} index={}", Uri, *Index);
	}

	/// Clear the entire navigation stack.
	pub fn Clear(&self) {
		let mut Stack = self.Stack.lock();
		let mut Index = self.Index.lock();

		Stack.clear();

		*Index = 0;
		dev_log!("history", "[NavigationHistoryState] Stack cleared");
	}

	/// Return all URIs in the stack (oldest first).
	pub fn GetStack(&self) -> Vec<String> { self.Stack.lock().clone() }
}