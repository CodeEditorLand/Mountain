//! `WorkingCopyState::SetDirty`

use super::Struct;
use std::{
	collections::HashSet,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::dev_log;

pub fn Fn(This:&Struct, Uri:&str, Dirty:bool) {
		if let Ok(mut Guard) = This.DirtyUris.lock() {
			if Dirty {
				Guard.insert(Uri.to_owned());

				dev_log!("workingcopy", "[WorkingCopyState] URI marked dirty: {}", Uri);
			} else {
				Guard.remove(Uri);

				dev_log!("workingcopy", "[WorkingCopyState] URI marked clean: {}", Uri);
			}
		}
	}
