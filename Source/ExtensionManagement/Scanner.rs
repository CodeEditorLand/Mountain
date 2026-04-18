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
//! ```text
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
//! - `process_configuration_properties`: Recursive configuration property
//! processor

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::{DTO::FileTypeDTO::FileTypeDTO, ReadDirectory::ReadDirectory, ReadFile::ReadFile},
};
use serde_json::{Map, Value};
use tauri::Manager;

use crate::{
	ApplicationState::{ApplicationState, DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO},
	Environment::Utility,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
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

	// Distinguish "directory does not exist" (first-run, no user extensions
	// installed yet — perfectly normal) from a real I/O failure. Only the
	// latter deserves a `warn:` prefix; the former is debug-level noise.
	match DirectoryPath.try_exists() {
		Ok(false) => {
			dev_log!(
				"extensions",
				"[ExtensionScanner] Extension path '{}' does not exist, skipping (no extensions installed here)",
				DirectoryPath.display()
			);
			return Ok(Vec::new());
		},
		Err(error) => {
			dev_log!(
				"extensions",
				"[ExtensionScanner] Could not stat extension path '{}': {} — skipping",
				DirectoryPath.display(),
				error
			);
			return Ok(Vec::new());
		},
		Ok(true) => {},
	}

	let TopLevelEntries = match RunTime.Run(ReadDirectory(DirectoryPath.clone())).await {
		Ok(entries) => entries,

		Err(error) => {
			dev_log!(
				"extensions",
				"warn: [ExtensionScanner] Could not read extension directory '{}': {}. Skipping.",
				DirectoryPath.display(),
				error
			);

			return Ok(Vec::new());
		},
	};

	dev_log!(
		"extensions",
		"[ExtensionScanner] Directory '{}' contains {} top-level entries",
		DirectoryPath.display(),
		TopLevelEntries.len()
	);

	let mut parse_failures = 0usize;
	let mut missing_package_json = 0usize;

	for (EntryName, FileType) in TopLevelEntries {
		if FileType == FileTypeDTO::Directory {
			let PotentialExtensionPath = DirectoryPath.join(EntryName);

			let PackageJsonPath = PotentialExtensionPath.join("package.json");

			dev_log!(
				"extensions",
				"[ExtensionScanner] Checking for package.json in: {}",
				PotentialExtensionPath.display()
			);

			match RunTime.Run(ReadFile(PackageJsonPath.clone())).await {
				Ok(PackageJsonContent) => {
					// Parse to a dynamic JSON value first so we can resolve
					// VS Code NLS placeholders (`%key%` strings referencing
					// `package.nls.json` entries) across every typed field.
					// Without this the UI renders literal `%command.clone%`,
					// `%displayName%`, etc. in the Command Palette and menus.
					let mut ManifestValue:Value = match serde_json::from_slice::<Value>(&PackageJsonContent) {
						Ok(v) => v,
						Err(error) => {
							parse_failures += 1;
							dev_log!(
								"extensions",
								"warn: [ExtensionScanner] Failed to parse package.json at '{}': {}",
								PotentialExtensionPath.display(),
								error
							);
							continue;
						},
					};

					if let Some(NLSMap) = LoadNLSBundle(&RunTime, &PotentialExtensionPath).await {
						let mut Replaced = 0u32;
						let mut Unresolved = 0u32;
						ResolveNLSPlaceholdersInner(&mut ManifestValue, &NLSMap, &mut Replaced, &mut Unresolved);
						dev_log!(
							"extensions",
							"[LandFix:NLS] {} → {} replaced, {} unresolved placeholders",
							PotentialExtensionPath.display(),
							Replaced,
							Unresolved
						);
					}

					match serde_json::from_value::<ExtensionDescriptionStateDTO>(ManifestValue) {
						Ok(mut Description) => {
							// Augment the description with its location on disk.
							Description.ExtensionLocation =
								serde_json::to_value(url::Url::from_directory_path(&PotentialExtensionPath).unwrap())
									.unwrap_or(Value::Null);

							// Construct identifier from publisher.name if not set
							if Description.Identifier == Value::Null
								|| Description.Identifier == Value::Object(Default::default())
							{
								let Id = if Description.Publisher.is_empty() {
									Description.Name.clone()
								} else {
									format!("{}.{}", Description.Publisher, Description.Name)
								};
								Description.Identifier = serde_json::json!({ "value": Id });
							}

							// Mark as built-in extension
							Description.IsBuiltin = true;

							FoundExtensions.push(Description);
						},

						Err(error) => {
							parse_failures += 1;
							dev_log!(
								"extensions",
								"warn: [ExtensionScanner] Failed to parse package.json for extension at '{}': {}",
								PotentialExtensionPath.display(),
								error
							);
						},
					}
				},
				Err(error) => {
					missing_package_json += 1;
					dev_log!(
						"extensions",
						"warn: [ExtensionScanner] Could not read package.json at '{}': {}",
						PackageJsonPath.display(),
						error
					);
				},
			}
		}
	}

	dev_log!(
		"extensions",
		"[ExtensionScanner] Directory '{}' scan done: {} parsed, {} parse-failures, {} missing package.json",
		DirectoryPath.display(),
		FoundExtensions.len(),
		parse_failures,
		missing_package_json
	);

	Ok(FoundExtensions)
}

/// Load an extension's NLS bundle (`package.nls.json`) into a `{key → string}`
/// map. Returns `None` if the bundle is absent or unreadable; placeholders stay
/// as-is in that case. Entries can be bare strings or `{message, comment}`
/// objects — we only keep `message`.
async fn LoadNLSBundle(RunTime:&Arc<ApplicationRunTime>, ExtensionPath:&PathBuf) -> Option<Map<String, Value>> {
	let NLSPath = ExtensionPath.join("package.nls.json");
	let Content = match RunTime.Run(ReadFile(NLSPath.clone())).await {
		Ok(Bytes) => Bytes,
		Err(Error) => {
			dev_log!(
				"extensions",
				"[LandFix:NLS] no bundle for {} ({})",
				ExtensionPath.display(),
				Error
			);
			return None;
		},
	};
	let Parsed:Value = match serde_json::from_slice(&Content) {
		Ok(V) => V,
		Err(Error) => {
			dev_log!(
				"extensions",
				"warn: [LandFix:NLS] failed to parse {}: {}",
				NLSPath.display(),
				Error
			);
			return None;
		},
	};
	let Object = Parsed.as_object()?;
	let mut Resolved = Map::with_capacity(Object.len());
	for (Key, RawValue) in Object {
		let Text = if let Some(s) = RawValue.as_str() {
			Some(s.to_string())
		} else if let Some(obj) = RawValue.as_object() {
			obj.get("message").and_then(|m| m.as_str()).map(|s| s.to_string())
		} else {
			None
		};
		if let Some(t) = Text {
			Resolved.insert(Key.clone(), Value::String(t));
		}
	}
	dev_log!(
		"extensions",
		"[LandFix:NLS] loaded {} keys for {}",
		Resolved.len(),
		ExtensionPath.display()
	);
	Some(Resolved)
}

/// Recursively walks a JSON `Value` tree and replaces every string of the form
/// `%key%` with the corresponding NLS entry. Mirrors VS Code's
/// `replaceNLStrings` in `src/vs/platform/extensionManagement/common/
/// extensionNls.ts`. Unknown keys are left untouched so UIs at least show the
/// key rather than nothing.
fn ResolveNLSPlaceholders(Value:&mut Value, NLS:&Map<String, Value>) {
	ResolveNLSPlaceholdersInner(Value, NLS, &mut 0u32, &mut 0u32);
}

/// Internal NLS walker that also counts substitutions made vs. unresolved
/// placeholders it saw, so the outer scanner can log a one-line summary per
/// extension.
fn ResolveNLSPlaceholdersInner(Value:&mut Value, NLS:&Map<String, Value>, Replaced:&mut u32, Unresolved:&mut u32) {
	match Value {
		serde_json::Value::String(Text) => {
			if Text.len() >= 2 && Text.starts_with('%') && Text.ends_with('%') {
				let Key = &Text[1..Text.len() - 1];
				if !Key.is_empty() && !Key.contains('%') {
					if let Some(Replacement) = NLS.get(Key).and_then(|v| v.as_str()) {
						*Text = Replacement.to_string();
						*Replaced += 1;
					} else {
						*Unresolved += 1;
					}
				}
			}
		},
		serde_json::Value::Array(Items) => {
			for Item in Items {
				ResolveNLSPlaceholdersInner(Item, NLS, Replaced, Unresolved);
			}
		},
		serde_json::Value::Object(Map) => {
			for (_, FieldValue) in Map {
				ResolveNLSPlaceholdersInner(FieldValue, NLS, Replaced, Unresolved);
			}
		},
		_ => {},
	}
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
