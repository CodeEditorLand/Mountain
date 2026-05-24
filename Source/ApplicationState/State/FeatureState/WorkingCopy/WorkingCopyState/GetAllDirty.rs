//! `WorkingCopyState::GetAllDirty`

use super::Struct;
use std::{
	collections::HashSet,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(This:&Struct) -> Vec<String> {
		This.DirtyUris
			.lock()
			.ok()
			.map(|Guard| Guard.iter().cloned().collect())
			.unwrap_or_default()
	}
