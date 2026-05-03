#![allow(non_snake_case)]

//! Flush the consecutive-duplicate buffer - emits a `(xN)` tail
//! line if the pending count is greater than 1, then clears.

use crate::IPC::DevLog::{DedupState, WriteToFile};

pub fn Fn() {
	if let Ok(mut State) = DedupState::DEDUP.lock() {
		if State.Count > 1 {
			let Tail = format!("  (x{})", State.Count);
			eprintln!("{}", Tail);
			WriteToFile::Fn(&Tail);
		}
		State.LastKey.clear();
		State.Count = 0;
	}
}
