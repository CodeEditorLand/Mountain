// ---------------------------------------------------------------------------------------------
// Mountain Storage Handlers (handlers/storage.rs)
// --------------------------------------------------------------------------------------------
// Implements the backend logic for the Extension Storage API (also known as
// Memento API in VS Code). This allows extensions running in sidecars (e.g.,

// Cocoon) to persistently store and retrieve key-value data, scoped either
// globally (shared across all workspaces) or to the current workspace.
//
// Responsibilities:
// - Handling `$getValue` and `$setValue` RPC calls, which are typically
//   transformed into effects by `track.rs` and then executed by the
//   `StorageProvider` implementation in `environment.rs`. The `StorageProvider`
//   then calls these handler functions.
// - Differentiating between Global (`StorageScope = 1`) and Workspace
//   (`StorageScope = 0`) storage.
// - Accessing and modifying in-memory storage maps (`global_memento`,

//   `workspace_memento`) held in `AppState`.
// - Implementing persistence:
//   - Loading storage data from disk during `AppState` initialization (see
//     `app_state.rs`).
//   - Asynchronously saving storage data to disk (`.json` files) whenever
//     `$setValue` is called.
// - Resolving file paths for persistent storage based on scope and workspace
//   ID.
//
// Key Interactions:
// - Primarily called by `environment.rs` (implementing `StorageProvider` trait
//   methods).
// - Interacts with `AppState` for accessing Memento HashMaps
//   (`AppState.global_memento`, `AppState.workspace_memento`) and their
//   respective file paths (`AppState.global_memento_path`,

//   `AppState.workspace_memento_path`).
// - Uses `tokio::fs` and `tokio::io::AsyncWriteExt` for asynchronous file
//   writing.
// - Uses `serde_json` for serializing and deserializing storage data to/from
//   JSON.
// - Uses `handlers::error_utils` for consistent RPC error formatting.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	path::{Path, PathBuf},

	// StdMutex for AppState fields
	sync::{Arc, Mutex as StdMutex, MutexGuard},
};

use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
// `State` is not directly used in RPC handler signatures here as `AppHandle` gives access.
use tauri::{AppHandle, Manager, Runtime};
// Tokio fs for async operations
use tokio::{fs, io::AsyncWriteExt};

use crate::app_state::AppState;
// Use shared error utilities
use crate::handlers::error_utils;

// CommonError is not directly returned by these RPC handlers; they return
// String. However, internal operations might map to/from CommonError.
// use Land_Common::errors::CommonError;

// --- Type Aliases ---

/// Type alias for storage scope identifier.
/// `0` represents Workspace-scoped storage.
/// `1` represents Global-scoped storage.
/// This is `pub` for use by `StorageProvider` in `environment.rs` and
/// `app_state.rs`.
// 0 = Workspace, 1 = Global
pub type StorageScope = u32;

/// Type alias for the in-memory representation of a Memento storage map.
/// Key: `String`, Value: `serde_json::Value`.
/// This is `pub` for use by `StorageProvider` in `environment.rs` and
/// `app_state.rs`.
pub type StorageMap = HashMap<String, Value>;

// --- Helper Functions ---

/// Formats a `PoisonError` from a Mutex lock on storage state into a
/// standardized RPC error string.
///
/// # Arguments
/// * `e` - The `PoisonError` encountered.
///
/// # Returns
/// A `String` containing a JSON-formatted RPC error.
fn format_storage_lock_error_for_rpc<T>(e:std::sync::PoisonError<MutexGuard<'_, T>>) -> String {
	let msg = format!("[Storage Handler LockErr] Failed to acquire lock on storage state: {}", e);

	// Log the specific error internally
	error!("{}", msg);

	// Specific lock error code
	error_utils::rpc_error_string(msg, Some("ELOCKED_STORAGE"))
}

/// Parses the storage scope (0 for Workspace, 1 for Global) and the storage key
/// (string) from the JSON `Value` parameters received via RPC.
///
/// # Arguments
/// * `params` - The `serde_json::Value` containing RPC parameters. Expected to
///   be an object with `scope` (number) and `key` (string) fields.
/// * `operation_name` - Name of the calling operation (e.g.,
///
///
///   "storage_getValue"), used for error reporting.
///
/// # Returns
/// * `Ok((StorageScope, String))` with the parsed scope and key.
/// * `Err(String)` containing a JSON-RPC error string if parsing fails or
///   parameters are invalid.
fn parse_storage_scope_and_key_from_params(
	params:&Value,

	operation_name:&str,
) -> Result<(StorageScope, String), String> {
	// Scope is expected as a number: 0 for Workspace, 1 for Global.
	// Assuming params is an object like { "scope": 0, "key": "myKey" }

	// This matches the structure in `storage_effects.rs`.
	let scope = params
		.get("scope")
		 // Parse as u64 first
		.and_then(Value::as_u64)
		 // Then cast to u32 (StorageScope)
		.map(|s| s as StorageScope)
		.ok_or_else(|| {

			error_utils::rpc_param_error_string(
				operation_name,



				 // Parameter name within the object
				"params.scope",


				"0 (Workspace) or 1 (Global) as number",


				 // Index not applicable for object fields
				None,


			)
		})?;

	if scope != 0 && scope != 1 {
		return Err(error_utils::rpc_error_string(
			"Invalid 'scope' value in parameters (must be 0 for Workspace or 1 for Global)".to_string(),
			Some("EBADARG_SCOPE"),
		));
	}

	let key = params
		.get("key")
		.and_then(Value::as_str)
		 // Ensure key is not empty
		.filter(|s| !s.is_empty())
		.map(String::from)
		.ok_or_else(|| {

			error_utils::rpc_param_error_string(
				operation_name,


				 // Parameter name within the object
				"params.key",


				"non-empty string",


				None,


			)
		})?;

	Ok((scope, key))
}

// --- Persistence Helper (Async) ---

/// Asynchronously saves the provided storage map (as JSON) to the specified
/// file path.
///
/// This function will create the parent directory of the `path` if it doesn't
/// already exist. It's made `pub` for potential use during application shutdown
/// or by `AppState` directly if needed for explicit save operations beyond the
/// automatic save-on-set.
///
/// # Arguments
/// * `path` - The `Path` to the file where the storage data should be saved.
/// * `data` - A reference to the `StorageMap` to be serialized and saved.
///
/// # Returns
/// * `Ok(())` on successful serialization and file write.
/// * `Err(String)` containing an error message if any step fails (e.g.,
///
///
///   serialization, directory creation, file write). This error string is not
///   JSON-RPC formatted as it's an internal utility.
pub async fn save_storage_map_to_disk(path:&Path, data:&StorageMap) -> Result<(), String> {
	let json_string = serde_json::to_string_pretty(data).map_err(|e| {
		format!(
			"[Storage Persistence] Failed to serialize storage data for path '{}': {}",
			path.display(),
			e
		)
	})?;

	if let Some(parent_dir) = path.parent() {
		// Use tokio::fs for asynchronous directory existence check and creation.
		if !tokio::fs::try_exists(parent_dir).await.unwrap_or(false) {
			info!(
				"[Storage Persistence] Creating parent storage directory: {}",
				parent_dir.display()
			);

			fs::create_dir_all(parent_dir).await.map_err(|e| {
				format!(
					"[Storage Persistence] Failed to create storage directory '{}': {}",
					parent_dir.display(),
					e
				)
			})?;
		}
	} else {
		// This case should be rare for valid paths but is a safeguard.
		return Err(format!(
			"[Storage Persistence] Invalid storage path (has no parent directory): {}",
			path.display()
		));
	}

	debug!(
		"[Storage Persistence] Writing storage state ({} keys) to file: {}",
		data.len(),
		path.display()
	);

	// Use tokio::fs::File and AsyncWriteExt for asynchronous file operations.
	let mut file = fs::File::create(path).await.map_err(|e| {
		format!(
			"[Storage Persistence] Failed to create/open storage file '{}' for writing: {}",
			path.display(),
			e
		)
	})?;

	file.write_all(json_string.as_bytes()).await.map_err(|e| {
		format!(
			"[Storage Persistence] Failed to write storage data to file '{}': {}",
			path.display(),
			e
		)
	})?;

	// Keep: Confirmation log for successful persistence.
	info!("[Storage Persistence] Successfully saved storage state to {}", path.display());

	Ok(())
}

// --- State Access Helper ---

/// Retrieves the appropriate storage map `Arc<StdMutex<StorageMap>>` and its
/// persistence file `PathBuf` from `AppState` based on the given `scope`.
///
/// This is a utility function made `pub` for use by `storage_effects.rs` (via
/// `StorageProvider` in `environment.rs`) and potentially by `app_state.rs`
/// during initialization or shutdown.
///
/// # Arguments
/// * `app_state` - A reference to the `AppState`.
/// * `scope` - The `StorageScope` (0 for Workspace, 1 for Global).
///
/// # Returns
/// * `Ok((Arc<StdMutex<StorageMap>>, Option<PathBuf>))` where the `PathBuf` is
///   `None` if the workspace path is not yet set (for workspace scope).
/// * `Err(String)` containing a JSON-RPC error string if a lock on
///   `workspace_memento_path` is poisoned.
pub fn get_storage_map_and_path_from_appstate(
	app_state:&AppState,

	scope:StorageScope,
) -> Result<(Arc<StdMutex<StorageMap>>, Option<PathBuf>), String> {
	let memento_map_mutex = if scope == 1 {
		// Global scope
		app_state.global_memento.clone()
	} else {
		// Workspace scope (scope == 0)
		app_state.workspace_memento.clone()
	};

	let memento_file_path_opt = if scope == 1 {
		Some(app_state.global_memento_path.clone())
	} else {
		// Workspace path is itself wrapped in Arc<StdMutex<Option<PathBuf>>>
		// Need to lock it to get the Option<PathBuf>.
		match app_state.workspace_memento_path.lock() {
			// Clone the Option<PathBuf> from within the guard
			Ok(guard) => guard.clone(),

			Err(e) => return Err(format_storage_lock_error_for_rpc(e)),
		}
	};

	Ok((memento_map_mutex, memento_file_path_opt))
}

// --- RPC Request Handlers (Called by effects or direct RPC dispatcher via
// Track) ---

/// Handles the `storage_getValue` request (typically via an effect).
///
/// Retrieves a value from the appropriate storage scope (Global or Workspace)
/// based on the provided key.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`.
/// * `params` - A `serde_json::Value` object expected to be `{ "scope": number,
///
///
///   "key": string }`.
///
/// # Returns
/// * `Ok(Value)` containing the retrieved value, or `Value::Null` if the key is
///   not found.
/// * `Err(String)` with a JSON-RPC error if parameters are invalid or state
///   access fails.
pub async fn handle_get_storage_value<R:Runtime>(
	app:AppHandle<R>,

	// Expects { "scope": 0 | 1, "key": "string" }
	params:Value,
) -> Result<Value, String> {
	let (scope, key) = parse_storage_scope_and_key_from_params(&params, "storage_getValue")?;

	let scope_name = if scope == 1 { "Global" } else { "Workspace" };

	// Reduce logging for frequent 'get' requests unless debugging.
	trace!("[Storage Handler] GetValue: scope={}, key='{}'", scope_name, key);

	let app_state = app.state::<AppState>();

	let (storage_map_mutex, _storage_file_path_opt) = get_storage_map_and_path_from_appstate(&app_state, scope)?;

	let storage_map_guard = storage_map_mutex.lock().map_err(format_storage_lock_error_for_rpc)?;

	// Return the cloned value if found, otherwise Value::Null.
	Ok(storage_map_guard.get(&key).cloned().unwrap_or(Value::Null))
}

/// Handles the `storage_setValue` request (typically via an effect).
///
/// Sets or deletes a value in the appropriate storage scope (Global or
/// Workspace). If `value_to_set` is `Value::Null`, the key is removed.
/// After updating the in-memory map, this function triggers an asynchronous
/// task to persist the changes to disk.
///
/// # Arguments
/// * `app` - The Tauri `AppHandle`.
/// * `params` - A `serde_json::Value` object expected to be `{ "scope": number,
///
///
///   "key": string, "value": any_json_value_or_null }`.
///
/// # Returns
/// * `Ok(Value::Null)` on successful update of the in-memory map (persistence
///   is async).
/// * `Err(String)` with a JSON-RPC error if parameters are invalid or state
///   access fails.
pub async fn handle_set_storage_value<R:Runtime>(
	app:AppHandle<R>,

	// Expects { "scope": 0 | 1, "key": "string", "value": any | null }
	params:Value,
) -> Result<Value, String> {
	let (scope, key) = parse_storage_scope_and_key_from_params(&params, "storage_setValue")?;

	// `value` can be null to delete the key.
	let value_to_set = params.get("value").cloned().ok_or_else(|| {
		error_utils::rpc_param_error_string(
			"storage_setValue",
			// Parameter name
			"params.value",
			"any JSON value or null to delete",
			None,
		)
	})?;

	let scope_name_str = if scope == 1 { "Global" } else { "Workspace" };

	// Keep log for set operations as they modify state.
	info!(
		"[Storage Handler] SetValue: scope={}, key='{}', value_is_null={}",
		scope_name_str,
		key,
		value_to_set.is_null()
	);

	trace!(
		"[Storage Handler] Value for key '{}' in scope '{}': {}...",
		key,
		scope_name_str,
		value_to_set
			.to_string()
			.chars()
			 // Log a sample of the value
			.take(100)
			.collect::<String>()
	);

	let app_state = app.state::<AppState>();

	let (storage_map_mutex, storage_file_path_opt) = get_storage_map_and_path_from_appstate(&app_state, scope)?;

	// Clone data needed for saving *after* the lock is released to minimize lock
	// duration.
	let data_clone_for_async_save:Option<StorageMap> = {
		let mut storage_map_guard = storage_map_mutex.lock().map_err(format_storage_lock_error_for_rpc)?;

		if value_to_set.is_null() {
			debug!("[Storage Handler] Deleting key '{}' from {} Memento.", key, scope_name_str);

			storage_map_guard.remove(&key);
		} else {
			storage_map_guard.insert(key.clone(), value_to_set);
		}

		// Clone the entire HashMap for saving only if a persistence path is available
		// for this scope.
		storage_file_path_opt.as_ref().map(|_| storage_map_guard.clone())
		// Mutex lock released here.
	};

	// Trigger asynchronous save task if a path and data are available.
	if let (Some(path_to_save_to), Some(data_to_persist)) = (storage_file_path_opt, data_clone_for_async_save) {
		// Clone path for the async task.
		let path_owned_for_task = path_to_save_to.clone();

		tokio::spawn(async move {
			debug!(
				"[Storage Handler Task] Persisting {} storage ({} keys) to: {}",
				scope_name_str,
				data_to_persist.len(),
				path_owned_for_task.display()
			);

			if let Err(e_str) = save_storage_map_to_disk(&path_owned_for_task, &data_to_persist).await {
				error!(
					"[Storage Handler Task] Error persisting {} storage to '{}': {}",
					scope_name_str,
					path_owned_for_task.display(),
					e_str
				);

				// TODO: Consider a mechanism for retrying failed saves or
				// notifying the user       if persistent storage is
				// critical and repeatedly failing.
			}
		});
	} else if scope != 1 && storage_file_path_opt.is_none() {
		// This case specifically handles Workspace scope where the memento path might
		// not be set (e.g., no workspace open, or error resolving path).
		warn!(
			"[Storage Handler] Workspace storage path not set. Cannot persist value for key '{}'. Change will only be \
			 in-memory for this session.",
			key
		);
	}

	// `setValue` operation in VS Code is void (returns undefined).
	// Return null on success of updating the in-memory map.
	Ok(Value::Null)
}
