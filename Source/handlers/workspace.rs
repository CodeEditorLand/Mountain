// ---------------------------------------------------------------------------------------------
// Mountain Workspace Handlers (handlers/workspace.rs)
// --------------------------------------------------------------------------------------------
// Handles RPC requests from Cocoon's `workspace-shim.js` related to workspace
// information, state, and file searching. It also includes logic for notifying
// Cocoon when workspace state changes within Mountain.
//
// Responsibilities:
// - Handling `$getWorkspaceFolders` RPC calls: Retrieves the current list of
//   `WorkspaceFolderState` from `AppState` and serializes them into the
//   expected `UriComponents` + name/index format.
// - Handling `$requestWorkspaceTrust` RPC calls: Returns the current trust
//   state from `AppState.is_trusted`.
// - Handling `$findFiles` RPC calls: Parses glob patterns and options, uses the
//   `ignore` crate to search files within workspace folders respecting ignore
//   files, and returns matching file URIs.
// - Providing internal helper functions (`notify_cocoon_of_folder_change`,
//   `notify_cocoon_of_trust_change`) to be called when Mountain modifies
//   workspace state, which then send notifications
//   (`$onDidChangeWorkspaceFolders`, `$onDidGrantWorkspaceTrust`) via Vine to
//   Cocoon.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` (or workspace effects).
// - Interacts with `AppState` to read workspace folders and trust state.
// - Uses `vine::send_notification` to push state changes to Cocoon.
// - Uses `ignore` and `globset` crates for `findFiles` implementation.
// - Relies on `WorkspaceFolderState` definition in `app_state.rs`.
// --------------------------------------------------------------------------------------------

use std::{
	path::PathBuf,
	sync::{
		Arc,
		Mutex as StdMutex,
		atomic::{AtomicBool, Ordering}, // Added for trust state
	},
};

use globset::{Glob, GlobMatcher}; // Added for findFiles
use ignore::WalkBuilder; // Added for findFiles, respecting ignore files
use log; // Use log crate
use serde::Deserialize; // Added for findFiles options deserialization
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime, State}; // State might be needed if runtime injected later
use url::Url; // Use Url for consistency

use crate::{
	app_state::{AppState, WorkspaceFolderState}, // Import AppState and nested structs
	vine,                                        // For sending notifications
};

// --- Helper Structs/Enums ---

/// Options for the `findFiles` operation, mirroring VS Code's API.
#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
struct FindFilesOptions {
	max_results:Option<usize>,
	use_ignore_files:Option<bool>,        // Respect .gitignore, .ignore?
	use_global_ignore_files:Option<bool>, // Respect global gitignore?
	use_parent_ignore_files:Option<bool>, // Respect ignore files in parent dirs? (Less common)
	follow_symlinks:Option<bool>,         /* Follow symbolic links?
	                                       * TODO: Add other options like `excludes` from settings? */
}

/// Represents the glob pattern parameter, which can be a simple string
/// or an object with `pattern` and optional `base` URI components.
#[derive(Deserialize, Debug)]
#[serde(untagged)] // Allows parsing string OR object into this enum
enum GlobParam {
	String(String),
	Pattern { pattern:String, base:Option<Value> }, // Base is expected as UriComponents JSON Value
}

// --- Helper Functions ---

/// Creates a structured error JSON string for RPC error responses.
fn create_error_string(message:String, code:Option<&str>) -> String {
	json!({
		 "message": message,
		 "code": code.unwrap_or("EUNKNOWN")
	})
	.to_string()
}

/// Helper to convert PathBuf to file UriComponents JSON Value.
fn path_to_uri_components(p:&PathBuf) -> Option<Value> {
	p.to_str().map(|s| {
		json!({
			"scheme": "file",
			"path": s,
			"external": format!("file://{}", s) // Include external string form
		})
	})
}

// --- Request Handlers (Called by Track dispatcher) ---

/// Handles the `workspace_getWorkspaceFolders` request from Cocoon.
/// Retrieves the current list of workspace folders from AppState.
pub async fn handle_get_workspace_folders<R:Runtime>(app:AppHandle<R>) -> Result<Value, String> {
	log::info!("[Workspace Handler] Handling getWorkspaceFolders request");
	let app_state = app.state::<AppState>();

	// Access the workspace folders stored in the AppState.
	let folders_lock = app_state.workspace_folders.lock().map_err(|e| {
		log::error!("Failed to acquire lock on workspace folders state: {}", e);
		create_error_string(format!("Internal error locking workspace state: {}", e), None)
	})?;

	// Serialize the folder data into the JSON format expected by VS Code API
	let folders_json:Vec<Value> = folders_lock
		.iter()
		.map(|folder:&WorkspaceFolderState| {
			json!({
				// Include $mid only if strictly necessary for VS Code marshalling
				// "$mid": 1,
				"scheme": folder.uri.scheme(),
				"authority": folder.uri.host_str(), // host_str() returns Option<&str>
				"path": folder.uri.path(),
				"query": folder.uri.query(),
				"fragment": folder.uri.fragment(),
				"name": folder.name,
				"index": folder.index,
				"external": folder.uri.to_string(), // Include external form
			})
		})
		.collect();

	// Drop lock explicitly before returning (good practice)
	drop(folders_lock);

	Ok(json!(folders_json)) // Return the JSON array
}

/// Handles the `workspace_getWorkspaceFolder` request from Cocoon (STUBBED).
/// Should retrieve a specific workspace folder based on a provided URI.
pub async fn handle_get_workspace_folder<R:Runtime>(
	app:AppHandle<R>,
	params:Value, // Expects URI components of the resource to check
) -> Result<Value, String> {
	log::warn!("[Workspace Handler] Handling getWorkspaceFolder request (STUBBED)");
	// TODO: Parse the target URI from `params`.
	// TODO: Implement the logic from `workspace-shim.js`'s getWorkspaceFolder
	//       on the native side using `AppState.workspace_folders`.
	// TODO: Serialize the found folder (matching the format in
	// handle_get_workspace_folders)       or return Value::Null if not found.
	Err(create_error_string(
		"getWorkspaceFolder not fully implemented".to_string(),
		Some("ENOSYS"),
	))
}

/// Handles the `workspace_requestTrust` request from Cocoon.
/// Returns the current workspace trust state from Mountain's AppState.
pub async fn handle_request_trust<R:Runtime>(
	app:AppHandle<R>,
	_params:Value, // Params might contain details about the trust request in future
) -> Result<Value, String> {
	log::info!("[Workspace Handler] Handling requestWorkspaceTrust request");
	let app_state = app.state::<AppState>();
	// For MVP, just return the current boolean state stored atomically.
	// A real implementation might involve checks or prompting the user via UI
	// effects.
	let is_trusted = app_state.is_trusted.load(Ordering::Relaxed);
	log::debug!("[Workspace Handler] Current trust state: {}", is_trusted);
	Ok(json!(is_trusted))
}

/// Handles `workspace_findFiles` request from Cocoon.
/// Performs a file search within the workspace using glob patterns and
/// respecting ignore files. Args: `[include: GlobParam, exclude?: GlobParam |
/// null, options?: FindFilesOptions]`
pub async fn handle_find_files<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	// --- Argument Parsing ---
	let params_array = params
		.as_array()
		.ok_or_else(|| create_error_string("Invalid parameters: Expected JSON array".to_string(), Some("EBADARG")))?;

	let include_param_val = params_array
		.get(0)
		.cloned()
		.ok_or_else(|| create_error_string("Missing 'include' pattern parameter".to_string(), Some("EBADARG")))?;
	let exclude_param_val = params_array.get(1).cloned(); // Optional
	let options_val = params_array.get(2).cloned(); // Optional

	// Deserialize parameters
	let include_param:GlobParam = serde_json::from_value(include_param_val)
		.map_err(|e| create_error_string(format!("Invalid include pattern: {}", e), Some("EBADARG")))?;

	let exclude_param_opt:Option<GlobParam> = exclude_param_val
		.filter(|v| !v.is_null()) // Treat null as None
		.map(serde_json::from_value)
		.transpose() // Convert Option<Result> to Result<Option>
		.map_err(|e| create_error_string(format!("Invalid exclude pattern: {}", e), Some("EBADARG")))?;

	let options:FindFilesOptions = options_val
		.map(serde_json::from_value)
		.transpose()
		.map_err(|e| create_error_string(format!("Invalid options object: {}", e), Some("EBADARG")))?
		.unwrap_or_default(); // Use default options if not provided or null

	log::info!(
		"[Workspace Handler] Handling findFiles request: include={:?}, exclude={:?}, options={:?}",
		include_param,
		exclude_param_opt,
		options
	);

	// --- Check Workspace Folders ---
	let app_state = app.state::<AppState>();
	let folders_guard = app_state.workspace_folders.lock().map_err(|e| {
		log::error!("Failed to lock workspace folders: {}", e);
		create_error_string("Internal error locking workspace state".to_string(), None)
	})?;

	if folders_guard.is_empty() {
		log::info!("[Workspace Handler] findFiles: No workspace folders open.");
		return Ok(json!([])); // No folders -> no results
	}

	// --- Build Glob Matchers ---
	// Helper to build GlobMatcher from GlobParam, handling potential base paths
	let build_matcher = |param:&GlobParam| -> Result<(GlobMatcher, Option<PathBuf>), String> {
		let (pattern_str, base_val_opt) = match param {
			GlobParam::String(s) => (s.as_str(), None),
			GlobParam::Pattern { pattern, base } => (pattern.as_str(), base.as_ref()),
		};

		// Parse base path if provided
		let base_path_opt = if let Some(base_val) = base_val_opt {
			// Reuse handler's URI component parsing logic (if made public or duplicated)
			// For now, assume it returns a PathBuf Result
			fn temp_path_from_uri(uri_val:&Value) -> Result<PathBuf, String> {
				// Simplified version of the helper in fs_api handlers
				let scheme = uri_val.get("scheme").and_then(|v| v.as_str()).unwrap_or("file");
				if scheme != "file" && !scheme.is_empty() {
					return Err("Base must be file scheme".into());
				}
				let path_str = uri_val.get("path").and_then(|v| v.as_str()).ok_or("Missing base path")?;
				Ok(PathBuf::from(path_str))
			}
			Some(temp_path_from_uri(base_val)?)
		} else {
			None
		};

		let glob = Glob::new(pattern_str).map_err(|e| {
			create_error_string(format!("Invalid glob pattern '{}': {}", pattern_str, e), Some("EBADGLOB"))
		})?; // Custom code for bad glob

		Ok((glob.compile_matcher(), base_path_opt))
	};

	let (include_matcher, include_base) = build_matcher(&include_param)?;
	let exclude_opt = exclude_param_opt.as_ref().map(build_matcher).transpose()?;
	let exclude_matcher = exclude_opt.as_ref().map(|(m, _b)| m); // We only need the matcher part
	// TODO: Handle exclude_base correctly if needed (rarely used with exclude?)

	// --- Perform Search ---
	let mut results:Vec<Value> = Vec::new();
	let max_results = options.max_results.unwrap_or(usize::MAX);

	// Iterate over each workspace folder
	for folder in folders_guard.iter() {
		let folder_root = PathBuf::from(folder.uri.path()); // Assumes file URI
		log::debug!("[Workspace Handler] Searching in folder: {}", folder_root.display());

		// Determine the effective search root (folder root or include base if specified
		// and inside folder)
		let search_root = include_base
			.as_ref()
			.filter(|base| base.starts_with(&folder_root)) // Ensure base is within folder
			.unwrap_or(&folder_root);

		// Configure the directory walker from the `ignore` crate
		let mut walker_builder = WalkBuilder::new(search_root);
		walker_builder.standard_filters(options.use_ignore_files.unwrap_or(true)); // Respect .gitignore, .ignore
		walker_builder.git_global(options.use_global_ignore_files.unwrap_or(true)); // Respect global gitignore
		walker_builder.git_ignore(options.use_ignore_files.unwrap_or(true));
		walker_builder.git_exclude(options.use_ignore_files.unwrap_or(true));
		walker_builder.follow_links(options.follow_symlinks.unwrap_or(false)); // Option to follow symlinks
		if let Some(parent_ignore) = options.use_parent_ignore_files {
			walker_builder.parents(parent_ignore); // Respect ignore files in parent dirs
		}

		// Walk the directory
		for result_entry in walker_builder.build() {
			if results.len() >= max_results {
				break;
			} // Stop if max results reached

			match result_entry {
				Ok(entry) => {
					let absolute_path = entry.path();
					// Skip the root directory itself if include base wasn't used explicitly
					if include_base.is_none() && absolute_path == folder_root {
						continue;
					}

					// Match against globs using path relative to the *folder root* for consistency
					if let Ok(relative_path) = absolute_path.strip_prefix(&folder_root) {
						if include_matcher.is_match(relative_path) {
							if exclude_matcher.map_or(false, |ex| ex.is_match(relative_path)) {
								continue; // Skip if excluded
							}

							// Convert absolute path back to file URI components
							if let Some(uri_components) = path_to_uri_components(&absolute_path.to_path_buf()) {
								results.push(uri_components);
							} else {
								log::warn!(
									"[Workspace Handler] findFiles: Failed to convert result path {} to file URI \
									 components",
									absolute_path.display()
								);
							}
						}
					} else {
						// This might happen for paths outside the folder root if symlinks are followed
						// excessively
						log::warn!(
							"[Workspace Handler] findFiles: Found path {} outside folder root {}",
							absolute_path.display(),
							folder_root.display()
						);
					}
				},
				Err(e) => {
					log::error!(
						"[Workspace Handler] findFiles: Error walking directory {}: {}",
						folder_root.display(),
						e
					)
				},
			}
		}
		if results.len() >= max_results {
			break;
		} // Stop iterating folders if max results reached
	}

	// Drop lock after iteration
	drop(folders_guard);

	log::info!("[Workspace Handler] findFiles found {} results.", results.len());
	Ok(json!(results)) // Return JSON array of UriComponents
}

// --- Notification Helpers (Called internally by Mountain) ---

/// Notifies Cocoon when workspace folders change.
/// Sends the `$onDidChangeWorkspaceFolders` notification via Vine.
pub async fn notify_cocoon_of_folder_change<R:Runtime>(app:AppHandle<R>) {
	log::info!("[Workspace Handler] Notifying Cocoon of workspace folder change");
	// Payload for this notification is typically empty according to VS Code
	// protocol, it just signals that the shim should re-request the folders.
	let notification_method = "$onDidChangeWorkspaceFolders".to_string();
	let sidecar_id = "cocoon-main"; // Target the main extension host sidecar

	// Send notification via Vine IPC.
	if let Err(e) = vine::send_notification(sidecar_id, notification_method, json!({})).await {
		log::error!(
			"[Workspace Handler] Failed to send folder change notification to {}: {}",
			sidecar_id,
			e
		);
	}
}

/// Notifies Cocoon when workspace trust state changes.
/// Sends the `$onDidGrantWorkspaceTrust` notification via Vine.
pub async fn notify_cocoon_of_trust_change<R:Runtime>(app:AppHandle<R>, _is_trusted:bool) {
	log::info!("[Workspace Handler] Notifying Cocoon of workspace trust change");
	// Payload for this notification is empty according to VS Code protocol.
	let notification_method = "$onDidGrantWorkspaceTrust".to_string();
	let sidecar_id = "cocoon-main";

	if let Err(e) = vine::send_notification(sidecar_id, notification_method, json!({})).await {
		log::error!(
			"[Workspace Handler] Failed to send trust change notification to {}: {}",
			sidecar_id,
			e
		);
	}
}

// Example usage within Mountain after state change:
// let handle = self.app_handle.clone();
// tokio::spawn(async move {
//     handlers::workspace::notify_cocoon_of_folder_change(handle).await;
// });
// let trust_state = app_state.is_trusted.load(Ordering::Relaxed);
// let handle = self.app_handle.clone();
// tokio::spawn(async move {
//     handlers::workspace::notify_cocoon_of_trust_change(handle,
// trust_state).await; });
