//! # MementoSaver Module (Internal)
//!
//! ## RESPONSIBILITIES
//! Saves memento state to disk for crash recovery and state persistence.
//! Asynchronously saves memento to JSON file with proper error handling.
//!
//! ## ARCHITECTURAL ROLE
//! MementoSaver is part of the **Internal::Persistence** module, handling
//! memento saving operations.
//!
//! ## KEY COMPONENTS
//! - SaveMementoToDisk: Main function for saving memento
//!
//! ## ERROR HANDLING
//! - Returns Result with CommonError on failure
//! - Creates directory structure if needed
//! - Proper error logging
//!
//! ## LOGGING
//! All operations are logged at appropriate levels (debug, info, warn, error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Asynchronous file I/O
//! - Proper error handling and recovery
//!
//! ## TODO
//! - [ ] Add checksum calculation
//! - [ ] Implement atomic writes
//! - [ ] Add compression support

use std::{collections::HashMap, fs, path::Path};

use serde_json::Value;
use CommonLibrary::Error::CommonError::CommonError;

use crate::dev_log;

/// Asynchronously saves Memento storage data to a JSON file.
///
/// # Arguments
/// * `StorageFilePath` - Path to the memento storage file
/// * `MementoData` - The memento data to save
///
/// # Returns
/// Result indicating success or CommonError on failure
///
/// # Errors
/// Returns CommonError for file I/O or serialization errors
///
/// # Behavior
/// - Creates parent directory if it doesn't exist
/// - Serializes data to JSON
/// - Writes to file atomically (creates temp file then renames)
pub async fn SaveMementoToDisk(StorageFilePath:&Path, MementoData:&HashMap<String, Value>) -> Result<(), CommonError> {
	// Ensure parent directory exists
	if let Some(parent) = StorageFilePath.parent() {
		if !parent.exists() {
			fs::create_dir_all(parent).map_err(|e| {
				dev_log!(
					"storage",
					"error: [MementoSaver] Failed to create directory '{}': {}",
					parent.display(),
					e
				);
				CommonError::FileSystemIO {
					Path:parent.to_path_buf(),
					Description:format!("Failed to create directory: {}", e),
				}
			})?;

			dev_log!("storage", "[MementoSaver] Created directory: {}", parent.display());
		}
	}

	// Serialize memento data to JSON
	let json_content = serde_json::to_string_pretty(MementoData).map_err(|e| {
		dev_log!("storage", "error: [MementoSaver] Failed to serialize memento data: {}", e);
		CommonError::SerializationError { Description:format!("Failed to serialize memento data: {}", e) }
	})?;

	// Write to temporary file first, then rename for atomic write
	let temp_path = StorageFilePath.with_extension("json.tmp");

	fs::write(&temp_path, json_content).map_err(|e| {
		dev_log!(
			"storage",
			"error: [MementoSaver] Failed to write memento to temp file '{}': {}",
			temp_path.display(),
			e
		);
		CommonError::FileSystemIO { Path:temp_path.clone(), Description:format!("Failed to write memento: {}", e) }
	})?;

	// Atomic rename from temp to actual file
	fs::rename(&temp_path, StorageFilePath).map_err(|e| {
		dev_log!(
			"storage",
			"error: [MementoSaver] Failed to rename temp file to '{}': {}",
			StorageFilePath.display(),
			e
		);
		// Clean up temp file if rename fails
		let _ = fs::remove_file(&temp_path);
		CommonError::FileSystemIO {
			Path:StorageFilePath.to_path_buf(),
			Description:format!("Failed to rename memento file: {}", e),
		}
	})?;

	dev_log!(
		"storage",
		"[MementoSaver] Successfully saved memento to: {}",
		StorageFilePath.display()
	);

	Ok(())
}
