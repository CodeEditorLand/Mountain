
//! Result-typed memento loader. Returns `Ok(empty)` for missing
//! files, `Err(FileSystemIO)` for read failures, and
//! `Err(SerializationError)` for parse failures (with a timestamped
//! corruption backup written as a side effect). Used during recovery
//! flows where the caller needs to know that loading actually
//! failed.

use std::{collections::HashMap, fs, path::Path};

use CommonLibrary::Error::CommonError::CommonError;
use serde_json::Value;

use crate::{ApplicationState::Internal::Persistence::MementoLoader::CreateCorruptedBackup, dev_log};

pub fn Fn(StorageFilePath:&Path) -> Result<HashMap<String, Value>, CommonError> {
	if !StorageFilePath.exists() {
		dev_log!(
			"storage",
			"[MementoLoader] Memento file does not exist: {}",
			StorageFilePath.display()
		);

		return Ok(HashMap::new());
	}

	let Content = fs::read_to_string(StorageFilePath).map_err(|E| {
		CommonError::FileSystemIO {
			Path:StorageFilePath.to_path_buf(),
			Description:format!("Failed to read memento file: {}", E),
		}
	})?;

	serde_json::from_str(&Content).map_err(|E| {
		CreateCorruptedBackup::Fn(StorageFilePath, &Content);
		CommonError::SerializationError {
			Description:format!("Failed to parse memento JSON from '{}': {}", StorageFilePath.display(), E),
		}
	})
}
