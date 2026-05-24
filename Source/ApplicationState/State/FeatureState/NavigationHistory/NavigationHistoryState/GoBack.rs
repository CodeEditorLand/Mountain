//! `NavigationHistoryState::GoBack`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use crate::dev_log;

pub fn Fn(This:&Struct) -> Option<String> {
		let Stack = This.Stack.lock().ok()?;

		if Stack.is_empty() {
			return None;
		}

		let mut Index = This.Index.lock().ok()?;

		if *Index == 0 {
			return None;
		}

		*Index -= 1;
		let Uri = Stack.get(*Index).cloned();

		dev_log!("history", "[NavigationHistoryState] GoBack → index={} uri={:?}", *Index, Uri);

		Uri
	}
