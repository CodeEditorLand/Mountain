// @module ScanLogic
// @description Contains the logic for scanning the filesystem for installed
// extensions and populating their metadata into the application's state.

use std::path::PathBuf;

use log::{info, trace, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use tokio::fs;

use crate::ApplicationState::{ApplicationState::ApplicationState, DTO::ExtensionDescriptionStateDto};

/// An internal helper to scan a single directory for extensions asynchronously.
async fn scan_single_directory(scan_dir:PathBuf) -> Vec<(String, ExtensionDescriptionStateDto)> {
	let mut found_extensions = Vec::new();
	if !scan_dir.is_dir() {
		return found_extensions;
	}

	let mut read_dir = match fs::read_dir(&scan_dir).await {
		Ok(rd) => rd,
		Err(e) => {
			warn!("[ExtensionScan] Failed to read directory '{}': {}", scan_dir.display(), e);
			return found_extensions;
		},
	};

	while let Ok(Some(entry)) = read_dir.next_entry().await {
		let package_json_path = entry.path().join("package.json");
		if package_json_path.is_file() {
			if let Ok(content) = fs::read_to_string(&package_json_path).await {
				if let Ok(package_json_value) = serde_json::from_str::<Value>(&content) {
					if let Ok(mut description) =
						serde_json::from_value::<ExtensionDescriptionStateDto>(package_json_value)
					{
						if let Some(id) = description.Identifier.get("value").and_then(Value::as_str) {
							if let Ok(location_uri) = url::Url::from_directory_path(entry.path()) {
								description.ExtensionLocation = serde_json::json!({ "scheme": "file", "path": entry.path(), "external": location_uri.to_string() });
								trace!("[ExtensionScan] Found extension: {}", id);
								found_extensions.push((id.to_string(), description));
							}
						}
					}
				}
			}
		}
	}
	found_extensions
}

/// Scans all configured paths for extensions in parallel and populates
/// `ApplicationState`.
pub async fn ScanExtensionsAndPopulateState<R:Runtime>(_app_handle:&AppHandle<R>, app_state:&ApplicationState) {
	let scan_paths = app_state.ExtensionScanPaths.lock().unwrap().clone();
	info!("[ExtensionScan] Starting parallel scan in paths: {:?}", scan_paths);

	let mut scan_tasks = Vec::new();
	for path in scan_paths {
		scan_tasks.push(tokio::spawn(scan_single_directory(path)));
	}

	let all_results = futures::future::join_all(scan_tasks).await;
	let mut final_extension_map = std::collections::HashMap::new();

	for result in all_results {
		if let Ok(extensions) = result {
			for (id, desc) in extensions {
				final_extension_map.insert(id, desc);
			}
		}
	}

	info!(
		"[ExtensionScan] Scan complete. Found {} total extensions.",
		final_extension_map.len()
	);
	*app_state.ScannedExtensions.lock().unwrap() = final_extension_map;
}

/// Initializes the list of paths to scan for extensions.
/// In a real app, this would read from configuration.
pub async fn InitializeScanPaths<R:Runtime>(app_handle:&AppHandle<R>, app_state:&ApplicationState) {
	let mut paths_to_scan = Vec::new();
	// Example: Add a built-in extensions directory.
	if let Some(resource_path) = app_handle.path_resolver().resolve_resource("extensions") {
		paths_to_scan.push(resource_path);
	}
	// Example: Add a user-specific extensions directory.
	if let Some(data_dir) = app_handle.path_resolver().app_data_dir() {
		let user_extensions_path = data_dir.join("extensions");
		if !user_extensions_path.exists() {
			tokio::fs::create_dir_all(&user_extensions_path).await.ok();
		}
		paths_to_scan.push(user_extensions_path);
	}
	*app_state.ExtensionScanPaths.lock().unwrap() = paths_to_scan;
}
