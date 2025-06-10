

/**
 * @module Internal (AppState)
 * @description Contains internal helper functions for the `AppState` module,
 * handling tasks like file I/O, path resolution, and serialization.
 */

#![allow(non_snake_case, non_camel_case_types)]

use log::{error, warn};
use serde::{self, Deserialize, Deserializer, Serializer};
use serde_json::Value;
use std::{collections::HashMap, path::Path};
use tokio::fs;
use url::Url;

/**
 * Analyzes text content to determine its line endings and splits it into a vector of lines.
 */
pub fn AnalyzeTextLinesAndEol(TextContent: &str) -> (Vec<String>, String) {
    let DetectedEol = if TextContent.contains("\r\n") { "\r\n" } else { "\n" };
    (TextContent.split(DetectedEol).map(String::from).collect(), DetectedEol.to_string())
}

/**
 * Asynchronously ensures the specified directory exists, creating it if necessary.
 */
pub async fn EnsureDirectoryExists(DirectoryPath: &Path) {
    if !DirectoryPath.exists() {
        if let Err(e) = fs::create_dir_all(DirectoryPath).await {
            error!("[AppState Internal] CRITICAL: Failed to create directory at '{}': {}.", DirectoryPath.display(), e);
        }
    }
}

/**
 * Asynchronously loads Memento storage data from a JSON file.
 */
pub async fn LoadInitialMementoFromDisk(StorageFilePath: &Path) -> HashMap<String, Value> {
    if !StorageFilePath.exists() {
        return HashMap::new();
    }
    match fs::read_to_string(StorageFilePath).await {
        Ok(Content) => serde_json::from_str(&Content).unwrap_or_else(|e| {
            error!("[AppState Internal] Failed to parse JSON from '{}': {}. Returning empty map.", StorageFilePath.display(), e);
            HashMap::new()
        }),
        Err(e) => {
            error!("[AppState Internal] Failed to read '{}': {}. Returning empty map.", StorageFilePath.display(), e);
            HashMap::new()
        }
    }
}

/**
 * Resolves the absolute path for a Memento storage file.
 */
pub fn ResolveMementoStorageFilePath(
    AppDataDirectory: &Path,
    IsGlobalScope: bool,
    WorkspaceIdentifier: &str,
) -> std::path::PathBuf {
    let UserStorageBasePath = AppDataDirectory.join("User");
    if IsGlobalScope {
        UserStorageBasePath.join("globalStorage.json")
    } else {
        // Sanitize the workspace identifier to be a safe directory name.
        let Segment = WorkspaceIdentifier.replace(|c: char| !c.is_alphanumeric() && c != '-' && c != '_', "_");
        UserStorageBasePath.join("workspaceStorage").join(Segment).join("storage.json")
    }
}

/**
 * A helper module for serializing and deserializing `url::Url` with `serde`.
 * This is used in DTOs where a `Url` field needs to be serialized to a string.
 */
pub mod UrlSerdeHelper {
    use super::*;
    pub fn serialize<S>(UrlInstance: &Url, SerializerInstance: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        SerializerInstance.serialize_str(UrlInstance.as_str())
    }

    pub fn deserialize<'de, D>(DeserializerInstance: D) -> Result<Url, D::Error>
    where
        D: Deserializer<'de>,
    {
        let StringValue = String::deserialize(DeserializerInstance)?;
        Url::parse(&StringValue).map_err(serde::de::Error::custom)
    }
}
