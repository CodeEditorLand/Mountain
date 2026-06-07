//! Timestamped corruption backup: write the failed-to-parse content
//! to a `.json.corrupted.YYYYMMDD_HHMMSS` sibling so several
//! recovery attempts in a row don't clobber each other. Pure
//! side-effect; never fails the caller.

use std::{fs, path::Path};

use crate::dev_log;

pub fn Fn(FilePath:&Path, Content:&str) {
	let Timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");

	let BackupPath = FilePath.with_extension(format!("json.corrupted.{}", Timestamp));

	if let Err(E) = fs::write(&BackupPath, Content) {
		dev_log!(
			"storage",
			"error: [MementoLoader] Failed to create corrupted backup at '{}': {}",
			BackupPath.display(),
			E
		);
	} else {
		dev_log!(
			"storage",
			"[MementoLoader] Created corrupted backup at: {}",
			BackupPath.display()
		);
	}
}
