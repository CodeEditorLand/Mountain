use std::path::PathBuf;

use log::{info, trace, warn};
use serde_json::Value;
use tauri::{ApplicationHandle, Manager, RunTime};
use tokio::fs;

// @module ScanLogic
// @description Contains the logic for scanning the filesystem for installed
// extensions and populating their metadata into the application's state.
use crate::ApplicationState::{ApplicationState::ApplicationState, DTO::ExtensionDescriptionStateDto};

// An internal helper to scan a single directory for extensions asynchronously.
async fn ScanSingleDirectory(ScanDir:PathBuf) -> Vec<(String, ExtensionDescriptionStateDto)> {
	let mut FoundExtensions = Vec::new();
	if !ScanDir.is_dir() {
		return FoundExtensions;
	}

	let mut ReadDir = match fs::read_dir(&ScanDir).await {
		Ok(rd) => rd,
		Err(e) => {
			warn!("[ExtensionScan] Failed to read directory '{}': {}", ScanDir.display(), e);
			return FoundExtensions;
		},
	};

	while let Ok(Some(Entry)) = ReadDir.next_entry().await {
		let PackageJsonPath = Entry.path().join("package.json");
		if PackageJsonPath.is_file() {
			if let Ok(Content) = fs::read_to_string(&PackageJsonPath).await {
				if let Ok(PackageJsonValue) = serde_json::from_str::<Value>(&Content) {
					if let Ok(mut Description) =
						serde_json::from_value::<ExtensionDescriptionStateDto>(PackageJsonValue)
					{
						if let Some(Id) = Description.Identifier.get("value").and_then(Value::as_str) {
							if let Ok(LocationUri) = url::Url::from_directory_path(Entry.path()) {
								Description.ExtensionLocation =
									serde_json::json!({ "external": LocationUri.to_string() });
								trace!("[ExtensionScan] Found extension: {}", Id);
								FoundExtensions.push((Id.to_string(), Description));
							}
						}
					}
				}
			}
		}
	}
	FoundExtensions
}

// Scans all configured paths for extensions in parallel and populates
// `ApplicationState`.
pub async fn ScanExtensionsAndPopulateState<R:RunTime>(_ApplicationHandle:&ApplicationHandle<R>, AppStateInstance:&ApplicationState) {
	let ScanPaths = AppStateInstance.ExtensionScanPaths.lock().unwrap().clone();
	info!("[ExtensionScan] Starting parallel scan in paths: {:?}", ScanPaths);

	let mut ScanTasks = Vec::new();
	for path in ScanPaths {
		ScanTasks.push(tokio::spawn(ScanSingleDirectory(path)));
	}

	let AllResults = futures::future::join_all(ScanTasks).await;
	let mut FinalExtensionMap = std::collections::HashMap::new();

	for result in AllResults {
		if let Ok(extensions) = result {
			for (id, desc) in extensions {
				FinalExtensionMap.insert(id, desc);
			}
		}
	}

	info!(
		"[ExtensionScan] Scan complete. Found {} total extensions.",
		FinalExtensionMap.len()
	);
	*AppStateInstance.ScannedExtensions.lock().unwrap() = FinalExtensionMap;
}

// Initializes the list of paths to scan for extensions.
// In a real app, this would read from configuration.
pub async fn InitializeScanPaths<R:RunTime>(ApplicationHandle:&ApplicationHandle<R>, AppStateInstance:&ApplicationState) {
	let mut PathsToScan = Vec::new();
	// Example: Add a built-in extensions directory.
	if let Some(ResourcePath) = ApplicationHandle.path_resolver().resolve_resource("extensions") {
		PathsToScan.push(ResourcePath);
	}
	// Example: Add a user-specific extensions directory.
	if let Some(DataDir) = ApplicationHandle.path_resolver().app_data_dir() {
		let UserExtensionsPath = DataDir.join("extensions");
		tokio::fs::create_dir_all(&UserExtensionsPath).await.ok();
		PathsToScan.push(UserExtensionsPath);
	}
	*AppStateInstance.ExtensionScanPaths.lock().unwrap() = PathsToScan;
}
