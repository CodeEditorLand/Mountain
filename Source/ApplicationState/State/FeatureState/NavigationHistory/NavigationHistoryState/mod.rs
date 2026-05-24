pub mod CanGoBack;
pub mod CanGoForward;
pub mod GoBack;
pub mod GoForward;
pub mod Push;
pub mod Clear;
pub mod GetStack;

use std::sync::{Arc, Mutex as StandardMutex};
use crate::dev_log;

/// Tracks the editor navigation history stack (back/forward).
/// Implements a cursor-based navigation stack where `Index` points to the
/// current position. GoBack decrements the index; GoForward increments it.
/// Pushing a new entry truncates forward history.
#[derive(Clone)]
pub struct Struct {
	/// The ordered list of visited URIs (oldest first).
	Stack:Arc<StandardMutex<Vec<String>>>,

	/// Current position in the stack (0-based). Points to the active entry.
	Index:Arc<StandardMutex<usize>>,
}
