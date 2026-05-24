//! `NavigationHistoryState::CanGoForward`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use crate::dev_log;

pub fn Fn(This:&Struct) -> bool {
		let Stack = This.Stack.lock().ok().map(|G| G.len()).unwrap_or(0);

		let Index = This.Index.lock().ok().as_deref().copied().unwrap_or(0);

		Stack > 0 && Index + 1 < Stack
	}
