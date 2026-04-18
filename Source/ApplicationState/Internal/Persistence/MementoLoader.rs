//! # MementoLoader Module (Internal)
//!
//! ## RESPONSIBILITIES
//! Loads memento state from disk for application initialization and recovery.
//! Synchronously loads memento from JSON file, handles missing files and
//! corrupted data.
//!
//! ## ARCHITECTURAL ROLE
//! MementoLoader is part of the **Internal::Persistence** module, handling
//! memento loading operations.
//!
//! ## KEY COMPONENTS
//! - LoadInitialMementoFromDisk: Main function for loading memento
//!
//! ## ERROR HANDLING
//! - Returns empty HashMap if file doesn't exist
//! - Creates backup and returns empty on parse error
//! - Creates directory on read error
//!
//! ## LOGGING
//! All operations are logged at appropriate levels (debug, info, warn, error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Synchronous file I/O for initialization
//! - Proper error handling and recovery
//!
//! ## TODO
//! - [ ] Add checksum validation
//! - [ ] Implement memento version migration
//! - [ ] Add incremental loading support

use std::{collections::HashMap, fs, path::Path};

use CommonLibrary::Error::CommonError::CommonError;
use serde_json::Value;

use crate::dev_log;

/// Synchronously loads Memento storage data from a JSON file.
/// Used during the initial `default()` setup of `ApplicationState`.
///
/// # Arguments
/// * `StorageFilePath` - Path to the memento storage file
///
/// # Returns
/// HashMap containing the memento data, or empty HashMap on error
///
/// # Behavior
/// - Returns empty HashMap if file doesn't exist
/// - Creates backup and returns empty on parse error
/// - Creates directory on read error
///
/// # Errors
/// Errors are logged but not propagated; default values are returned.
pub fn LoadInitialMementoFromDisk(StorageFilePath:&Path) -> HashMap<String, Value> {
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

				// Attempt recovery by creating backup and returning empty map
				attempt_memento_recovery(StorageFilePath, &Content);
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

			// Attempt recovery by ensuring directory exists
			if let Some(parent) = StorageFilePath.parent() {
				if !parent.exists() {
					if let Err(dir_error) = fs::create_dir_all(parent) {
						dev_log!(
							"storage",
							"warn: [MementoLoader] Failed to create directory '{}': {}",
							parent.display(),
							dir_error
						);
					}
				}
			}

			HashMap::new()
		},
	}
}

/// Robust memento loading with comprehensive error handling.
///
/// # Arguments
/// * `StorageFilePath` - Path to the memento storage file
///
/// # Returns
/// Result containing the memento HashMap or CommonError
///
/// # Errors
/// Returns CommonError for file I/O or parse errors
pub fn LoadMementoWithRecovery(StorageFilePath:&Path) -> Result<HashMap<String, Value>, CommonError> {
	if !StorageFilePath.exists() {
		dev_log!(
			"storage",
			"[MementoLoader] Memento file does not exist: {}",
			StorageFilePath.display()
		);
		return Ok(HashMap::new());
	}

	let content = fs::read_to_string(StorageFilePath).map_err(|e| {
		CommonError::FileSystemIO {
			Path:StorageFilePath.to_path_buf(),
			Description:format!("Failed to read memento file: {}", e),
		}
	})?;

	serde_json::from_str(&content).map_err(|e| {
		// Create backup of corrupted file
		create_corrupted_backup(StorageFilePath, &content);
		CommonError::SerializationError {
			Description:format!("Failed to parse memento JSON from '{}': {}", StorageFilePath.display(), e),
		}
	})
}

/// Attempt recovery for corrupted memento files.
///
/// # Arguments
/// * `file_path` - Path to the corrupted memento file
/// * `corrupted_content` - The corrupted content to backup
fn attempt_memento_recovery(file_path:&Path, corrupted_content:&str) {
	let backup_path = file_path.with_extension("json.backup");

	match fs::write(&backup_path, corrupted_content) {
		Ok(()) => {
			dev_log!(
				"storage",
				"warn: [MementoLoader] Created backup of corrupted memento at: {}",
				backup_path.display()
			);
		},
		Err(e) => {
			dev_log!(
				"storage",
				"error: [MementoLoader] Failed to create backup of corrupted memento at '{}': {}",
				backup_path.display(),
				e
			);
		},
	}
}

/// Create backup of corrupted file with timestamp.
///
/// # Arguments
/// * `file_path` - Path to the corrupted file
/// * `content` - The content to backup
fn create_corrupted_backup(file_path:&Path, content:&str) {
	let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
	let backup_path = file_path.with_extension(format!("json.corrupted.{}", timestamp));

	if let Err(e) = fs::write(&backup_path, content) {
		dev_log!(
			"storage",
			"error: [MementoLoader] Failed to create corrupted backup at '{}': {}",
			backup_path.display(),
			e
		);
	} else {
		dev_log!(
			"storage",
			"[MementoLoader] Created corrupted backup at: {}",
			backup_path.display()
		);
	}
}
