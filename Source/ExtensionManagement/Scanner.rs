// File: Mountain/Source/ExtensionManagement/Scanner.rs
// Role: Contains logic for discovering and parsing installed extensions.
// Responsibilities:
//   - Scan directories on the filesystem for extensions by reading
//     `package.json`.
//   - Collect and merge default configurations from all discovered extensions.

//! # Extension Scanner
//!
//! Contains the logic for scanning directories on the filesystem to discover
//! installed extensions by reading their `package.json` manifests.

#![allow(non_snake_case, non_camel_case_types)]

use std::{path::PathBuf, sync::Arc};

use Common::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::{DTO::FileTypeDTO::FileTypeDTO, ReadDirectory::ReadDirectory, ReadFile::ReadFile},
};
use log::{trace, warn};
use serde_json::{Map, Value};
use tauri::Manager;

use crate::{
	ApplicationState::{
		ApplicationState::ApplicationState,
		DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
	},
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
		.ScannedExtensions
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	for Extension in Extensions.values() {
		if let Some(contributes) = Extension.Contributes.as_ref().and_then(|v| v.as_object()) {
			if let Some(configuration) = contributes.get("configuration").and_then(|v| v.as_object()) {
				if let Some(properties) = configuration.get("properties").and_then(|v| v.as_object()) {
					// ADVANCED NESTED OBJECT HANDLING: Recursively process configuration properties
					self::process_configuration_properties(
						&mut MergedDefaults,
						"",
						properties,
						&mut Vec::new()
					)?;
				}
			}
		}
	}

	Ok(Value::Object(MergedDefaults))
}

/// ADVANCED RECURSIVE CONFIGURATION PROCESSING: Handle nested object structures
fn process_configuration_properties(
	merged_defaults: &mut serde_json::Map<String, Value>,
	current_path: &str,
	properties: &serde_json::Map<String, Value>,
	visited_keys: &mut Vec<String>
) -> Result<(), String> {
	for (key, value) in properties {
		// Build the full path for this property
		let full_path = if current_path.is_empty() {
			key.clone()
		} else {
			format!("{}.{}", current_path, key)
		};
		
		// Check for circular references
		if visited_keys.contains(&full_path) {
			return Err(format!("Circular reference detected in configuration properties: {}", full_path));
		}
		
		visited_keys.push(full_path.clone());
		
		if let Some(prop_details) = value.as_object() {
			// Check if this is a nested object structure
			if let Some(nested_properties) = prop_details.get("properties").and_then(|v| v.as_object()) {
				// Recursively process nested properties
				self::process_configuration_properties(
					merged_defaults,
					&full_path,
					nested_properties,
					visited_keys
				)?;
			} else if let Some(default_value) = prop_details.get("default") {
				// Handle regular property with default value
				merged_defaults.insert(full_path, default_value.clone());
			}
		}
		
		// Remove current key from visited keys
		visited_keys.retain(|k| k != &full_path);
	}
	
	Ok(())
}
