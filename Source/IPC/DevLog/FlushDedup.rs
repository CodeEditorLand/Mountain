
//! Flush the consecutive-duplicate buffer - emits a `(xN)` tail
//! line if the pending count is greater than 1, then clears.

use crate::IPC::DevLog::{DedupState, WriteToFile};

pub fn Fn() {
	// Use `try_lock` instead of `lock` so a contended flush (another
	// dev_log! call holding the mutex) simply skips the dedup tail rather
	// than parking the Tokio worker thread. The dedup compression is
	// cosmetic; missing one flush never loses a log line.
	let Ok(mut State) = DedupState::DEDUP.try_lock() else {
		return;
	};

	if State.Count > 1 {
		let Tail = format!("  (x{})", State.Count);

		eprintln!("{}", Tail);

		WriteToFile::Fn(&Tail);
	}

	State.LastKey.clear();

	State.Count = 0;
}
