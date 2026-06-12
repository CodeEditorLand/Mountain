//! Side-channel: write the corrupted memento payload to a `.backup`
//! sibling so a human can inspect the original. Failure to write the
//! backup is logged but doesn't propagate - the load path stays
//! best-effort.

use std::{fs, path::Path};

use crate::dev_log;

/// fn.
pub fn Fn(FilePath:&Path, CorruptedContent:&str) {
	let BackupPath = FilePath.with_extension("json.backup");

	match fs::write(&BackupPath, CorruptedContent) {
		Ok(()) => {
			dev_log!(
				"storage",
				"warn: [MementoLoader] Created backup of corrupted memento at: {}",
				BackupPath.display()
			)
		},

		Err(E) => {
			dev_log!(
				"storage",
				"error: [MementoLoader] Failed to create backup of corrupted memento at '{}': {}",
				BackupPath.display(),
				E
			)
		},
	}
}
