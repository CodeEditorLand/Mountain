//! `WorkingCopyState::IsDirty`

use super::Struct;
use std::{
	collections::HashSet,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(This:&Struct, Uri:&str) -> bool {
		This.DirtyUris.lock().ok().map(|Guard| Guard.contains(Uri)).unwrap_or(false)
	}
