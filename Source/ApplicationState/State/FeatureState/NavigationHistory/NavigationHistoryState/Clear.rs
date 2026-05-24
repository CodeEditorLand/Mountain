//! `NavigationHistoryState::Clear`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use crate::dev_log;

pub fn Fn(This:&Struct) {
		if let (Ok(mut Stack), Ok(mut Index)) = (This.Stack.lock(), This.Index.lock()) {
			Stack.clear();

			*Index = 0;
			dev_log!("history", "[NavigationHistoryState] Stack cleared");
		}
	}
