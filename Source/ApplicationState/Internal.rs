// File: Mountain/Source/ApplicationState/Internal.rs
// Role: Contains internal helper functions for the `ApplicationState` module.
// Responsibilities:
//   - Handle tasks like file I/O, path resolution, and serialization.
//   - Provide helper logic for populating state, like scanning for extensions.
//   - These are not part of the public API of the state struct itself.

//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # Internal (ApplicationState)
//!
//! Contains internal helper functions for the `ApplicationState` module,
//! handling tasks like file I/O, path resolution, and serialization that are
//! not part of the public API of the state itself.

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, fs, path::Path};

use Common::Error::CommonError::CommonError;
use log::{error, info, trace, warn, debug};
use serde::{self, Deserializer, Serializer};
use serde_json::Value;
use url::Url;

use crate::{
	ApplicationState::{
		ApplicationState::MapLockError,
		DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
	},
	ExtensionManagement,
};

/// Analyzes text content to determine its line endings and splits it into a
/// vector of lines.
pub fn AnalyzeTextLinesAndEOL(TextContent:&str) -> (Vec<String>, String) {
	let DetectedEOL = if TextContent.contains("\r\n") { "\r\n" } else { "\n" };

	(
		TextContent.split(DetectedEOL).map(String::from).collect(),
		DetectedEOL.to_string(),
	)
}

/// Synchronously loads Memento storage data from a JSON file.
/// Used during the initial `default()` setup of `ApplicationState`.
pub fn LoadInitialMementoFromDisk(StorageFilePath:&Path) -> HashMap<String, Value> {
	if !StorageFilePath.exists() {
		debug!("[AppStateInternal] Memento file does not exist: {}", StorageFilePath.display());
		return HashMap::new();
	}

	match fs::read_to_string(StorageFilePath) {
		Ok(Content) => {
			serde_json::from_str(&Content).unwrap_or_else(|Error| {
				error!(
					"[AppStateInternal] Failed to parse JSON from '{}': {}. Attempting recovery.",
					StorageFilePath.display(),
					Error
				);

				// Attempt recovery by creating backup and returning empty map
				attempt_memento_recovery(StorageFilePath, &Content);
				HashMap::new()
			})
		},

		Err(Error) => {
			error!(
				"[AppStateInternal] Failed to read '{}': {}. Attempting recovery.",
				StorageFilePath.display(),
				Error
			);

			// Attempt recovery by ensuring directory exists
			if let Some(parent) = StorageFilePath.parent() {
				if !parent.exists() {
					if let Err(dir_error) = fs::create_dir_all(parent) {
						warn!(
							"[AppStateInternal] Failed to create directory '{}': {}",
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

/// Robust memento loading with comprehensive error handling and recovery
pub fn LoadMementoWithRecovery(StorageFilePath:&Path) -> Result<HashMap<String, Value>, CommonError> {
	if !StorageFilePath.exists() {
		debug!("[AppStateInternal] Memento file does not exist: {}", StorageFilePath.display());
		return Ok(HashMap::new());
	}

	let content = fs::read_to_string(StorageFilePath).map_err(|e| {
		CommonError::FileSystemIO {
			Path: StorageFilePath.to_path_buf(),
			Description: format!("Failed to read memento file: {}", e),
		}
	})?;

	serde_json::from_str(&content).map_err(|e| {
		// Create backup of corrupted file
		create_corrupted_backup(StorageFilePath, &content);
		CommonError::SerializationError {
			Description: format!("Failed to parse memento JSON from '{}': {}", StorageFilePath.display(), e),
		}
	})
}

/// Attempt recovery for corrupted memento files
fn attempt_memento_recovery(file_path: &Path, corrupted_content: &str) {
	let backup_path = file_path.with_extension("json.backup");
	
	match fs::write(&backup_path, corrupted_content) {
		Ok(()) => {
			warn!(
				"[AppStateInternal] Created backup of corrupted memento at: {}",
				backup_path.display()
			);
		},
		Err(e) => {
			error!(
				"[AppStateInternal] Failed to create backup of corrupted memento at '{}': {}",
				backup_path.display(),
				e
			);
		},
	}
}

/// Create backup of corrupted file
fn create_corrupted_backup(file_path: &Path, content: &str) {
	let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
	let backup_path = file_path.with_extension(format!("json.corrupted.{}", timestamp));
	
	if let Err(e) = fs::write(&backup_path, content) {
		error!(
			"[AppStateInternal] Failed to create corrupted backup at '{}': {}",
			backup_path.display(),
			e
		);
	} else {
		debug!(
			"[AppStateInternal] Created corrupted backup at: {}",
			backup_path.display()
		);
	}
}

/// Resolves the absolute path for a Memento storage file based on scope.
pub fn ResolveMementoStorageFilePath(
	ApplicationDataDirectory:&Path,

	IsGlobalScope:bool,

	WorkSpaceIdentifier:&str,
) -> std::path::PathBuf {
	let UserStorageBasePath = ApplicationDataDirectory.join("User");

	if IsGlobalScope {
		UserStorageBasePath.join("globalStorage.json")
	} else {
		// Sanitize the workspace identifier to be a safe directory name.
		let Segment = WorkSpaceIdentifier.replace(|c:char| !c.is_alphanumeric() && c != '-' && c != '_', "_");

		UserStorageBasePath.join("workspaceStorage").join(Segment).join("storage.json")
	}
}

/// Scans all registered extension paths for valid extensions and populates the
/// state.
pub async fn ScanAndPopulateExtensions(
	ApplicationHandle:tauri::AppHandle,

	State:&crate::ApplicationState::ApplicationState::ApplicationState,
) -> Result<(), CommonError> {
	info!("[AppStateInternal] Starting extension scan...");

	let mut AllFoundExtensions:HashMap<String, ExtensionDescriptionStateDTO> = HashMap::new();

	let ScanPaths = State.ExtensionScanPaths.lock().map_err(|e| {
		crate::ApplicationState::ApplicationState::MapLockErrorWithRecovery(e, "ScanAndPopulateExtensions - ExtensionScanPaths")
	})?.clone();

	trace!("[AppStateInternal] Scanning paths: {:?}", ScanPaths);

	let mut successful_scans = 0;
	let mut failed_scans = 0;

	for Path in ScanPaths {
		match ExtensionManagement::Scanner::ScanDirectoryForExtensions(ApplicationHandle.clone(), Path).await {
			Ok(FoundInPath) => {
				successful_scans += 1;
				for Extension in FoundInPath {
					let Identifier = Extension
						.Identifier
						.get("value")
						.and_then(Value::as_str)
						.unwrap_or_default()
						.to_string();

					if !Identifier.is_empty() {
						AllFoundExtensions.insert(Identifier, Extension);
					}
				}
			},
			Err(error) => {
				failed_scans += 1;
				warn!(
					"[AppStateInternal] Failed to scan extension path '{}': {}",
					Path.display(),
					error
				);
			},
		}
	}

	let mut ScannedExtensionsGuard = State.ScannedExtensions.lock().map_err(|e| {
		crate::ApplicationState::ApplicationState::MapLockErrorWithRecovery(e, "ScanAndPopulateExtensions - ScannedExtensions")
	})?;

	*ScannedExtensionsGuard = AllFoundExtensions;

	info!(
		"[AppStateInternal] Extension scan complete. Found {} extensions ({} successful scans, {} failed scans).",
		ScannedExtensionsGuard.len(),
		successful_scans,
		failed_scans
	);

	if failed_scans > 0 {
		warn!("[AppStateInternal] {} extension paths failed to scan", failed_scans);
	}

	Ok(())
}

/// Robust extension scanning with comprehensive error handling
pub async fn ScanExtensionsWithRecovery(
	ApplicationHandle:tauri::AppHandle,
	State:&crate::ApplicationState::ApplicationState::ApplicationState,
) -> Result<(), CommonError> {
	info!("[AppStateInternal] Starting robust extension scan with recovery...");

	// Clear potentially corrupted extension state first
	let mut scanned_extensions = State.ScannedExtensions.lock().map_err(|e| {
		crate::ApplicationState::ApplicationState::MapLockErrorWithRecovery(e, "ScanExtensionsWithRecovery - Clear")
	})?;
	scanned_extensions.clear();
	drop(scanned_extensions);

	// Perform the scan
	match ScanAndPopulateExtensions(ApplicationHandle, State).await {
		Ok(()) => {
			info!("[AppStateInternal] Robust extension scan completed successfully");
			Ok(())
		},
		Err(error) => {
			error!("[AppStateInternal] Robust extension scan failed: {}", error);
			// Attempt recovery by clearing state and retrying once
			warn!("[AppStateInternal] Attempting recovery from extension scan failure...");
			
			// Clear state again
			let mut scanned_extensions = State.ScannedExtensions.lock().map_err(|e| {
				crate::ApplicationState::ApplicationState::MapLockErrorWithRecovery(e, "ScanExtensionsWithRecovery - Recovery Clear")
			})?;
			scanned_extensions.clear();
			drop(scanned_extensions);

			// Retry the scan
			ScanAndPopulateExtensions(ApplicationHandle, State).await
		},
	}
}

/// A helper module for serializing and deserializing `url::Url` with `serde`.
/// This is used in DTOs where a `Url` field needs to be serialized to a string.
pub mod URLSerializationHelper {

	use serde::Deserialize;

	use super::*;

	pub fn serialize<S>(URLInstance:&Url, SerializerInstance:S) -> Result<S::Ok, S::Error>
	where
		S: Serializer, {
		SerializerInstance.serialize_str(URLInstance.as_str())
	}

	pub fn deserialize<'de, D>(DeserializerInstance:D) -> Result<Url, D::Error>
	where
		D: Deserializer<'de>, {
		let StringValue = String::deserialize(DeserializerInstance)?;

		Url::parse(&StringValue).map_err(serde::de::Error::custom)
	}
}

/// State recovery utilities
pub mod RecoveryUtilities {
	use super::*;

	/// Validate and clean up state data
	pub fn validate_and_clean_state<T>(state_data: &mut HashMap<String, T>, validator: impl Fn(&T) -> bool) {
		state_data.retain(|_, value| validator(value));
	}

	/// Safe state operation with timeout
	pub fn safe_state_operation_with_timeout<T, F>(
		operation: F,
		timeout_ms: u64,
		operation_name: &str,
	) -> Result<T, CommonError>
	where
		F: FnOnce() -> Result<T, CommonError>,
	{
		let (sender, receiver) = std::sync::mpsc::channel();
		
		std::thread::spawn(move || {
			let result = operation();
			let _ = sender.send(result);
		});

		match receiver.recv_timeout(std::time::Duration::from_millis(timeout_ms)) {
			Ok(result) => result,
			Err(_) => {
				error!("[RecoveryUtilities] Operation '{}' timed out after {}ms", operation_name, timeout_ms);
				Err(CommonError::Unknown {
					Description: format!("Operation '{}' timed out", operation_name),
				})
			},
		}
	}

	/// Attempt state recovery with exponential backoff
	pub async fn recover_state_with_backoff<F, T>(
		operation: F,
		max_attempts: u32,
		operation_name: &str,
	) -> Result<T, CommonError>
	where
		F: Fn() -> Result<T, CommonError> + Send,
	{
		let mut attempt = 0;
		let mut delay_ms = 100;

		while attempt < max_attempts {
			match operation() {
				Ok(result) => return Ok(result),
				Err(error) => {
					attempt += 1;
					if attempt == max_attempts {
						return Err(error);
					}

					warn!(
						"[RecoveryUtilities] Attempt {} failed for '{}': {}. Retrying in {}ms...",
						attempt,
						operation_name,
						error,
						delay_ms
					);

					tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
					delay_ms *= 2; // Exponential backoff
				},
			}
		}

		Err(CommonError::Unknown {
			Description: format!("Failed to recover state for '{}' after {} attempts", operation_name, max_attempts),
		})
	}
}
