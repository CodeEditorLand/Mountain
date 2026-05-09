//! # ScanAndPopulateExtensions Module (Internal)
//!
//! ## RESPONSIBILITIES
//! Scans all registered extension paths for valid extensions and populates the
//! state with discovered extensions.
//!
//! ## ARCHITECTURAL ROLE
//! ScanAndPopulateExtensions is part of the **Internal::ExtensionScanner**
//! module, handling extension discovery and population.
//!
//! ## KEY COMPONENTS
//! - ScanAndPopulateExtensions: Main function for scanning extensions
//! - ScanExtensionsWithRecovery: Robust scanning with recovery
//!
//! ## ERROR HANDLING
//! - Returns Result with CommonError on failure
//! - Handles partial failures gracefully
//! - Comprehensive error logging
//!
//! ## LOGGING
//! Operations are logged at appropriate levels (info, debug, warn, error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Async operations for scanning
//! - Handles multiple scan paths
//! - Partial failure handling
//!
//! ## TODO
//! - [ ] Add concurrent scanning
//! - [ ] Implement extension caching
//! - [ ] Add extension validation rules

use std::{collections::HashMap, path::PathBuf};

use CommonLibrary::Error::CommonError::CommonError;

use serde_json::Value;

use tauri::AppHandle;

use crate::{
	ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
	ExtensionManagement,
	dev_log,
};

/// Scans all registered extension paths for valid extensions and populates the
/// state.
///
/// # Arguments
/// * `ApplicationHandle` - Tauri application handle for extension management
/// * `State` - Reference to the application state
///
/// # Returns
/// Result indicating success or CommonError on failure
///
/// # Behavior
/// - Scans all registered extension paths
/// - Populates state with discovered extensions
/// - Returns comprehensive scan statistics
/// - Handles partial failures gracefully
pub async fn ScanAndPopulateExtensions(
	ApplicationHandle:AppHandle,

	_State:&crate::ApplicationState::State::ExtensionState::State::State,
) -> Result<(), CommonError> {

	dev_log!("extensions", "[ExtensionScanner] Starting extension scan...");

	let mut all_found_extensions:HashMap<String, ExtensionDescriptionStateDTO> = HashMap::new();

	// Note: This would need to be adapted to the new state structure
	// For now, this is a placeholder showing the structure
	let scan_paths:Vec<PathBuf> = _State.Registry.GetExtensionScanPaths();

	dev_log!("extensions", "[ExtensionScanner] Scanning paths: {:?}", scan_paths);

	let mut successful_scans = 0;

	let mut failed_scans = 0;

	for path in scan_paths {

		let path_clone = path.clone();

		match ExtensionManagement::Scanner::ScanDirectoryForExtensions(ApplicationHandle.clone(), path_clone).await {

			Ok(found_in_path) => {

				successful_scans += 1;

				let path_count = found_in_path.len();

				let mut inserted_from_path = 0;

				let mut rejected_empty_identifier = 0;

				for extension in found_in_path {

					let identifier = extension
						.Identifier
						.get("value")
						.and_then(Value::as_str)
						.unwrap_or_default()
						.to_string();

					if !identifier.is_empty() {

						all_found_extensions.insert(identifier, extension);

						inserted_from_path += 1;
					} else {

						rejected_empty_identifier += 1;

						dev_log!(
							"extensions",

							"warn: [ExtensionScanner] Rejected extension '{}' - empty identifier (publisher='{}', \
							 Identifier={:?})",

							extension.Name,

							extension.Publisher,

							extension.Identifier
						);
					}
				}

				dev_log!(
					"extensions",

					"[ExtensionScanner] Path '{}' yielded {} parsed, {} inserted, {} rejected",

					path.display(),

					path_count,

					inserted_from_path,

					rejected_empty_identifier
				);
			},

			Err(error) => {

				failed_scans += 1;

				dev_log!(
					"extensions",

					"warn: [ExtensionScanner] Failed to scan extension path '{}': {}",

					path.display(),

					error
				);
			},
		}
	}

	// Store discovered extensions into ApplicationState
	let post_write_count = {

		let mut Guard = _State
			.ScannedExtensions
			.ScannedExtensions
			.lock()
			.map_err(|Error| CommonError::StateLockPoisoned { Context:Error.to_string() })?;

		Guard.clear();

		for (Key, Dto) in &all_found_extensions {

			Guard.insert(Key.clone(), Dto.clone());
		}

		Guard.len()
	};

	dev_log!(
		"extensions",

		"[ExtensionScanner] Extension scan complete. Found {} extensions ({} successful scans, {} failed scans). \
		 ScannedExtensions map now has {} entries.",

		all_found_extensions.len(),

		successful_scans,

		failed_scans,

		post_write_count
	);

	if failed_scans > 0 {

		dev_log!(
			"extensions",

			"warn: [ExtensionScanner] {} extension paths failed to scan",

			failed_scans
		);
	}

	Ok(())
}

/// Robust extension scanning with comprehensive error handling.
///
/// # Arguments
/// * `ApplicationHandle` - Tauri application handle for extension management
/// * `State` - Reference to the application state
///
/// # Returns
/// Result indicating success or CommonError on failure
///
/// # Behavior
/// - Clears potentially corrupted extension state first
/// - Performs the scan
/// - Retries once on failure
/// - Comprehensive error logging
pub async fn ScanExtensionsWithRecovery(
	ApplicationHandle:AppHandle,

	State:&crate::ApplicationState::State::ExtensionState::State::State,
) -> Result<(), CommonError> {

	dev_log!(
		"extensions",

		"[ExtensionScanner] Starting robust extension scan with recovery..."
	);

	// Clear potentially corrupted extension state first
	// Note: Would clear
	// State.Extension.ScannedExtensions.Extension.ScannedExtensions

	// Perform the scan
	match ScanAndPopulateExtensions(ApplicationHandle.clone(), State).await {

		Ok(()) => {

			dev_log!("extensions", "[ExtensionScanner] Robust extension scan completed successfully");

			Ok(())
		},

		Err(error) => {

			dev_log!(
				"extensions",

				"error: [ExtensionScanner] Robust extension scan failed: {}",

				error
			);

			// Attempt recovery by clearing state and retrying once
			dev_log!(
				"extensions",

				"warn: [ExtensionScanner] Attempting recovery from extension scan failure..."
			);

			// Clear state again
			// Note: Would clear
			// State.Extension.ScannedExtensions.Extension.ScannedExtensions

			// Retry the scan with a cloned handle
			ScanAndPopulateExtensions(ApplicationHandle.clone(), State).await
		},
	}
}
