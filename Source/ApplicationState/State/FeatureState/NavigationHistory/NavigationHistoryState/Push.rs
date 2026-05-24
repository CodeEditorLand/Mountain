//! `NavigationHistoryState::Push`

use super::Struct;
use std::sync::{Arc, Mutex as StandardMutex};
use crate::dev_log;

pub fn Fn(This:&Struct, Uri:String) {
		if let (Ok(mut Stack), Ok(mut Index)) = (This.Stack.lock(), This.Index.lock()) {
			// Truncate forward history
			let NewIndex = if Stack.is_empty() { 0 } else { *Index + 1 };

			Stack.truncate(NewIndex);

			Stack.push(Uri.clone());

			*Index = Stack.len() - 1;
			dev_log!("history", "[NavigationHistoryState] Push uri={} index={}", Uri, *Index);
		}
	}
