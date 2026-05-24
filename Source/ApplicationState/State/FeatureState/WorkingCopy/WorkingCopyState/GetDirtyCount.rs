//! `WorkingCopyState::GetDirtyCount`

use super::Struct;
use std::{
	collections::HashSet,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(This:&Struct) -> usize { This.DirtyUris.lock().ok().map(|Guard| Guard.len()).unwrap_or(0) }
