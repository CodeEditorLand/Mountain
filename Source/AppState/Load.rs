// File: AppState/Load.rs
// Defines a helper function for loading Memento storage data from a file on
// disk.

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, fs, path::Path};

use log::{debug, error, info};
use serde_json::Value;

// Type alias for clarity, representing the structure of Memento storage.
type MementoStorageMap = HashMap<String, Value>;

/// Loads Memento storage data from a JSON file at the specified path.
/// If the file does not exist or fails to parse, it returns an empty map.
pub fn LoadInitialMementoStorageFromDisk(StorageFilePath:&Path) -> MementoStorageMap {
	if !StorageFilePath.exists() {
		debug!(
			"[MementoLoad] Storage file not found: '{}'. Returning empty map.",
			StorageFilePath.display()
		);
		return HashMap::new();
	}
	debug!("[MementoLoad] Loading memento from: {}", StorageFilePath.display());
	match fs::read_to_string(StorageFilePath) {
		Ok(Content) => {
			if Content.trim().is_empty() {
				debug!(
					"[MementoLoad] Storage file '{}' is empty. Returning empty map.",
					StorageFilePath.display()
				);
				return HashMap::new();
			}
			match serde_json::from_str(&Content) {
				Ok(Map) => {
					info!("[MementoLoad] Loaded {} items from: {}", Map.len(), StorageFilePath.display());
					Map
				},
				Err(Error) => {
					error!(
						"[MementoLoad] Failed to parse JSON from '{}': {}. Returning empty map.",
						StorageFilePath.display(),
						Error
					);
					HashMap::new()
				},
			}
		},
		Err(Error) => {
			if Error.kind() != std::io::ErrorKind::NotFound {
				error!(
					"[MementoLoad] Failed to read '{}': {}. Returning empty map.",
					StorageFilePath.display(),
					Error
				);
			} else {
				debug!(
					"[MementoLoad] Storage file confirmed not found during read: {}",
					StorageFilePath.display()
				);
			}
			HashMap::new()
		},
	}
}
