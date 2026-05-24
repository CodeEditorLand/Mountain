pub mod IsDirty;
pub mod SetDirty;
pub mod GetAllDirty;
pub mod GetDirtyCount;

use std::{
	collections::HashSet,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

/// Tracks which URIs have unsaved changes (dirty state).
/// Drives the dirty dot in editor tabs and the explorer badge count.
#[derive(Clone)]
pub struct Struct {
	DirtyUris:Arc<StandardMutex<HashSet<String>>>,
}
