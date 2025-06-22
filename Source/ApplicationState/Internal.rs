// File: Mountain/Source/ApplicationState/Internal.rs
// Role: Contains internal helper functions for the `ApplicationState` module.
// Responsibilities:
//   - Handle tasks like file I/O, path resolution, and serialization.
//   - Provide helper logic for populating state, like scanning for extensions.
//   - These are not part of the public API of the state struct itself.

//! # Internal (ApplicationState)
//!
//! Contains internal helper functions for the `ApplicationState` module,

//! handling tasks like file I/O, path resolution, and serialization that are
//! not part of the public API of the state itself.

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, fs, path::Path};

use Common::Error::CommonError::CommonError;
use log::{error, info};
use serde::{self, Deserializer, Serializer};
use serde_json::Value;
use url::Url;

use crate::{ApplicationState::DTO::ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO, ExtensionManagement};

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
		return HashMap::new();
	}

	match fs::read_to_string(StorageFilePath) {
		Ok(Content) => {
			serde_json::from_str(&Content).unwrap_or_else(|e| {
				error!(
					"[AppStateInternal] Failed to parse JSON from '{}': {}. Returning empty map.",
					StorageFilePath.display(),
					e
				);

				HashMap::new()
			})
		},

		Err(e) => {
			error!(
				"[AppStateInternal] Failed to read '{}': {}. Returning empty map.",
				StorageFilePath.display(),
				e
			);

			HashMap::new()
		},
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

	let ScanPaths = State.ExtensionScanPaths.lock().unwrap().clone();

	for Path in ScanPaths {
		let FoundInPath =
			ExtensionManagement::Scanner::ScanDirectoryForExtensions(ApplicationHandle.clone(), Path).await?;

		for Extension in FoundInPath {
			let Identifier = Extension
				.Identifier
				.get("value")
				.and_then(Value::as_str)
				.unwrap_or_default()
				.to_string();

			AllFoundExtensions.insert(Identifier, Extension);
		}
	}

	let mut ScannedExtensionsGuard = State.ScannedExtensions.lock().unwrap();

	*ScannedExtensionsGuard = AllFoundExtensions;

	info!(
		"[AppStateInternal] Extension scan complete. Found {} extensions.",
		ScannedExtensionsGuard.len()
	);

	Ok(())
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
