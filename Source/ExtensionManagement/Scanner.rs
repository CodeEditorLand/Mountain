//! # Extension Scanner (ExtensionManagement)
//!
//! Contains the logic for scanning directories on the filesystem to discover
//! installed extensions by reading their `package.json` manifests, and for
//! collecting default configuration values from all discovered extensions.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Extension Discovery
//! - Scan registered extension paths for valid extensions
//! - Read and parse `package.json` manifest files
//! - Validate extension metadata and structure
//! - Build `ExtensionDescriptionStateDTO` for each discovered extension
//!
//! ### 2. Configuration Collection
//! - Extract default configuration values from extension
//!   `contributes.configuration`
//! - Merge configuration properties from all extensions
//! - Handle nested configuration objects recursively
//! - Detect and prevent circular references
//!
//! ### 3. Error Handling
//! - Gracefully handle unreadable directories
//! - Skip extensions with invalid package.json
//! - Log warnings for partial scan failures
//! - Continue scanning even when some paths fail
//!
//! ## ARCHITECTURAL ROLE
//!
//! The Extension Scanner is part of the **Extension Management** subsystem:
//!
//! ```
//! Startup ──► ScanPaths ──► Scanner ──► Extensions Map ──► ApplicationState
//! ```
//!
//! ### Position in Mountain
//! - `ExtensionManagement` module: Extension discovery and metadata
//! - Used during application startup to populate extension registry
//! - Provides data to `Cocoon` for extension host initialization
//!
//! ### Dependencies
//! - `CommonLibrary::FileSystem`: ReadDirectory and ReadFile effects
//! - `CommonLibrary::Error::CommonError`: Error handling
//! - `ApplicationRunTime`: Effect execution
//! - `ApplicationState`: Extension storage
//!
//! ### Dependents
//! - `InitializationData::ConstructExtensionHostInitializationData`: Sends
//!   extensions to Cocoon
//! - `MountainEnvironment::ScanForExtensions`: Public API for extension
//!   scanning
//! - `ApplicationState::Internal::ScanExtensionsWithRecovery`: Robust scanning
//!   wrapper
//!
//! ## SCANNING PROCESS
//!
//! 1. **Path Resolution**: Get scan paths from
//!    `ApplicationState.Extension.Registry.ExtensionScanPaths`
//! 2. **Directory Enumeration**: For each path, read directory entries
//! 3. **Manifest Detection**: Look for `package.json` in each subdirectory
//! 4. **Parsing**: Deserialize `package.json` into
//!    `ExtensionDescriptionStateDTO`
//! 5. **Augmentation**: Add `ExtensionLocation` (disk path) to metadata
//! 6. **Storage**: Insert into `ApplicationState.Extension.ScannedExtensions`
//!    map
//!
//! ## CONFIGURATION MERGING
//!
//! `CollectDefaultConfigurations()` extracts default values from all
//! extensions' `contributes.configuration.properties` and merges them into a
//! single JSON object:
//!
//! - Handles nested `.` notation (e.g., `editor.fontSize`)
//! - Recursively processes nested `properties` objects
//! - Detects circular references to prevent infinite loops
//! - Returns a flat map of configuration keys to default values
//!
//! ## ERROR HANDLING
//!
//! - **Directory Read Failures**: Logged as warnings, scanning continues
//! - **Invalid package.json**: Skipped with warning, scanning continues
//! - **IO Errors**: Logged, operation continues or fails gracefully
//!
//! ## PERFORMANCE
//!
//! - Scans are performed asynchronously via `ApplicationRunTime`
//! - Each directory read is a separate filesystem operation
//! - Large extension directories may impact startup time
//! - Consider caching scan results for development workflows
//!
//! ## VS CODE REFERENCE
//!
//! Borrowed from VS Code's extension management:
//! - `vs/workbench/services/extensions/common/extensionPoints.ts` -
//!   Configuration contribution
//! - `vs/platform/extensionManagement/common/extensionManagementService.ts` -
//!   Extension scanning
//!
//! ## TODO
//!
//! - [ ] Implement concurrent scanning for multiple paths
//! - [ ] Add extension scan caching with invalidation
//! - [ ] Implement extension validation rules (required fields, etc.)
//! - [ ] Add scan progress reporting for UI feedback
//! - [ ] Support extension scanning in subdirectories (recursive)
//!
//! ## MODULE CONTENTS
//!
//! - [`ScanDirectoryForExtensions`]: Scan a single directory for extensions
//! - [`CollectDefaultConfigurations`]: Merge configuration defaults from all
//!   extensions
//! - [`process_configuration_properties`]: Recursive configuration property
//!   processor

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::{DTO::FileTypeDTO::FileTypeDTO, ReadDirectory::ReadDirectory, ReadFile::ReadFile},
};
use log::{trace, warn};
use serde_json::{Map, Value};
use tauri::Manager;

use crate::{
	ApplicationState::{ApplicationState, DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO},
	Environment::Utility,
	RunTime::ApplicationRunTime::ApplicationRunTime,
};

/// Scans a single directory for valid extensions.
///
/// This function iterates through a given directory, looking for subdirectories
/// that contain a `package.json` file. It then attempts to parse this file
/// into an `ExtensionDescriptionStateDTO`.
pub async fn ScanDirectoryForExtensions(
	ApplicationHandle:tauri::AppHandle,

	DirectoryPath:PathBuf,
) -> Result<Vec<ExtensionDescriptionStateDTO>, CommonError> {
	let RunTime = ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let mut FoundExtensions = Vec::new();

	let TopLevelEntries = match RunTime.Run(ReadDirectory(DirectoryPath.clone())).await {
		Ok(entries) => entries,

		Err(error) => {
			warn!(
				"[ExtensionScanner] Could not read extension directory '{}': {}. Skipping.",
				DirectoryPath.display(),
				error
			);

			return Ok(Vec::new());
		},
	};

	for (EntryName, FileType) in TopLevelEntries {
		if FileType == FileTypeDTO::Directory {
			let PotentialExtensionPath = DirectoryPath.join(EntryName);

			let PackageJsonPath = PotentialExtensionPath.join("package.json");

			trace!(
				"[ExtensionScanner] Checking for package.json in: {}",
				PotentialExtensionPath.display()
			);

			if let Ok(PackageJsonContent) = RunTime.Run(ReadFile(PackageJsonPath)).await {
				match serde_json::from_slice::<ExtensionDescriptionStateDTO>(&PackageJsonContent) {
					Ok(mut Description) => {
						// Augment the description with its location on disk.
						Description.ExtensionLocation =
							serde_json::to_value(url::Url::from_directory_path(PotentialExtensionPath).unwrap())
								.unwrap_or(Value::Null);

						FoundExtensions.push(Description);
					},

					Err(error) => {
						warn!(
							"[ExtensionScanner] Failed to parse package.json for extension at '{}': {}",
							PotentialExtensionPath.display(),
							error
						);
					},
				}
			}
		}
	}

	Ok(FoundExtensions)
}

/// A helper function to extract default configuration values from all
/// scanned extensions.
pub fn CollectDefaultConfigurations(State:&ApplicationState) -> Result<Value, CommonError> {
	let mut MergedDefaults = Map::new();

	let Extensions = State
		.Extension
		.ScannedExtensions
		.ScannedExtensions
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	for Extension in Extensions.values() {
		if let Some(contributes) = Extension.Contributes.as_ref().and_then(|v| v.as_object()) {
			if let Some(configuration) = contributes.get("configuration").and_then(|v| v.as_object()) {
				if let Some(properties) = configuration.get("properties").and_then(|v| v.as_object()) {
					// NESTED OBJECT HANDLING: Recursively process configuration properties
					self::process_configuration_properties(&mut MergedDefaults, "", properties, &mut Vec::new())?;
				}
			}
		}
	}

	Ok(Value::Object(MergedDefaults))
}

/// RECURSIVE CONFIGURATION PROCESSING: Handle nested object structures
fn process_configuration_properties(
	merged_defaults:&mut serde_json::Map<String, Value>,
	current_path:&str,
	properties:&serde_json::Map<String, Value>,
	visited_keys:&mut Vec<String>,
) -> Result<(), CommonError> {
	for (key, value) in properties {
		// Build the full path for this property
		let full_path = if current_path.is_empty() {
			key.clone()
		} else {
			format!("{}.{}", current_path, key)
		};

		// Check for circular references
		if visited_keys.contains(&full_path) {
			return Err(CommonError::Unknown {
				Description:format!("Circular reference detected in configuration properties: {}", full_path),
			});
		}

		visited_keys.push(full_path.clone());

		if let Some(prop_details) = value.as_object() {
			// Check if this is a nested object structure
			if let Some(nested_properties) = prop_details.get("properties").and_then(|v| v.as_object()) {
				// Recursively process nested properties
				self::process_configuration_properties(merged_defaults, &full_path, nested_properties, visited_keys)?;
			} else if let Some(default_value) = prop_details.get("default") {
				// Handle regular property with default value
				merged_defaults.insert(full_path.clone(), default_value.clone());
			}
		}

		// Remove current key from visited keys
		visited_keys.retain(|k| k != &full_path);
	}

	Ok(())
}
