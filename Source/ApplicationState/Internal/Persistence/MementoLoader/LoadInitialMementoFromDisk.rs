#![allow(non_snake_case)]

//! Best-effort synchronous memento loader for `ApplicationState`'s
//! `default()` path. Failures don't propagate - corrupted files are
//! backed up, missing directories are created, and an empty map is
//! returned so initialisation always succeeds.

use std::{collections::HashMap, fs, path::Path};

use serde_json::Value;

use crate::{ApplicationState::Internal::Persistence::MementoLoader::AttemptMementoRecovery, dev_log};

pub fn Fn(StorageFilePath:&Path) -> HashMap<String, Value> {
	if !StorageFilePath.exists() {
		dev_log!(
			"storage",
			"[MementoLoader] Memento file does not exist: {}",
			StorageFilePath.display()
		);

		return HashMap::new();
	}

	match fs::read_to_string(StorageFilePath) {
		Ok(Content) => {
			serde_json::from_str(&Content).unwrap_or_else(|Error| {
				dev_log!(
					"storage",
					"error: [MementoLoader] Failed to parse JSON from '{}': {}. Attempting recovery.",
					StorageFilePath.display(),
					Error
				);
				AttemptMementoRecovery::Fn(StorageFilePath, &Content);
				HashMap::new()
			})
		},

		Err(Error) => {
			dev_log!(
				"storage",
				"error: [MementoLoader] Failed to read '{}': {}. Attempting recovery.",
				StorageFilePath.display(),
				Error
			);

			if let Some(Parent) = StorageFilePath.parent()
				&& !Parent.exists()
				&& let Err(DirError) = fs::create_dir_all(Parent)
			{
				dev_log!(
					"storage",
					"warn: [MementoLoader] Failed to create directory '{}': {}",
					Parent.display(),
					DirError
				);
			}

			HashMap::new()
		},
	}
}
