// @module Internal (ApplicationState)
// @description Contains internal helper functions for the `ApplicationState`
// module, handling tasks like file I/O, path resolution, and serialization.
//

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, fs, path::Path};

use log::{error, warn};
use serde::{self, Deserialize, Deserializer, Serializer};
use serde_json::Value;
use url::Url;

// Analyzes text content to determine its line endings and splits it into a
// vector of lines.
pub fn AnalyzeTextLinesAndEol(TextContent:&str) -> (Vec<String>, String) {
	let detected_eol = if TextContent.contains("\r\n") { "\r\n" } else { "\n" };
	(
		TextContent.split(detected_eol).map(String::from).collect(),
		detected_eol.to_string(),
	)
}

// Asynchronously ensures the specified directory exists, creating it if
// necessary.
pub async fn EnsureDirectoryExists(directory_path:&Path) {
	if !directory_path.exists() {
		if let Err(e) = tokio::fs::create_dir_all(directory_path).await {
			error!(
				"[ApplicationState Internal] CRITICAL: Failed to create directory at '{}': {}.",
				directory_path.display(),
				e
			);
		}
	}
}

// Synchronously loads Memento storage data from a JSON file. Used during
// initial `default()` setup.
pub fn LoadInitialMementoFromDisk(storage_file_path:&Path) -> HashMap<String, Value> {
	if !storage_file_path.exists() {
		return HashMap::new();
	}
	match fs::read_to_string(storage_file_path) {
		Ok(content) => {
			serde_json::from_str(&content).unwrap_or_else(|e| {
				error!(
					"[ApplicationState Internal] Failed to parse JSON from '{}': {}. Returning empty map.",
					storage_file_path.display(),
					e
				);
				HashMap::new()
			})
		},
		Err(e) => {
			error!(
				"[ApplicationState Internal] Failed to read '{}': {}. Returning empty map.",
				storage_file_path.display(),
				e
			);
			HashMap::new()
		},
	}
}

// Resolves the absolute path for a Memento storage file.
pub fn ResolveMementoStorageFilePath(
	app_data_directory:&Path,
	is_global_scope:bool,
	workspace_identifier:&str,
) -> std::path::PathBuf {
	let user_storage_base_path = app_data_directory.join("User");
	if is_global_scope {
		user_storage_base_path.join("globalStorage.json")
	} else {
		// Sanitize the workspace identifier to be a safe directory name.
		let segment = workspace_identifier.replace(|c:char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
		user_storage_base_path
			.join("workspaceStorage")
			.join(segment)
			.join("storage.json")
	}
}

// A helper module for serializing and deserializing `url::Url` with `serde`.
// This is used in DTOs where a `Url` field needs to be serialized to a string.
pub mod UrlSerdeHelper {
	use super::*;
	pub fn serialize<S>(url_instance:&Url, serializer_instance:S) -> Result<S::Ok, S::Error>
	where
		S: Serializer, {
		serializer_instance.serialize_str(url_instance.as_str())
	}

	pub fn deserialize<'de, D>(deserializer_instance:D) -> Result<Url, D::Error>
	where
		D: Deserializer<'de>, {
		let string_value = String::deserialize(deserializer_instance)?;
		Url::parse(&string_value).map_err(serde::de::Error::custom)
	}
}
