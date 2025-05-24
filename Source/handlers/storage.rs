// ---------------------------------------------------------------------------------------------
// Mountain Storage Handlers (handlers/storage.rs)

// --------------------------------------------------------------------------------------------
// Implements the backend logic for the Extension Storage API (Memento),

// handling persistent key-value storage for extensions running in sidecars
// (Cocoon).
//
// Responsibilities:
// - Handling `$getValue` and `$setValue` RPC calls proxied from Cocoon's
//   `storage-shim.js`.
// - Differentiating between Global (1) and Workspace (0) storage scopes based
//   on parameters.
// - Accessing the corresponding in-memory storage maps (`global_memento`,

//   `workspace_memento`) within the managed `AppState`.
// - Implementing basic persistence:
//   - Loading initial state from disk (e.g., JSON files) into `AppState` on
//     startup (handled in app_state.rs using `load_storage_from_disk`).
//   - Triggering asynchronous saving of modified storage maps back to disk
//     after `$setValue` using `save_storage_to_disk`.
// - Resolving appropriate file paths for persistence based on scope and
//   workspace context.
// - Handling file I/O errors and lock contention gracefully.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` for RPC methods.
// - Interacts with `AppState` via Mutex to access/modify Memento HashMaps and
//   storage paths.
// - Uses `tokio::fs` and `tokio::io::AsyncWriteExt` for asynchronous file
//   writing.
// - Uses `serde_json` for serializing/deserializing storage state.
// --------------------------------------------------------------------------------------------

// Use StdMutex if used directly in AppState
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{Arc, Mutex as StdMutex, MutexGuard},
};

// Use the log crate for logging
use log;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime, State};
// Use tokio fs/io for async operations
use tokio::{fs, io::AsyncWriteExt};

// Import AppState
use crate::app_state::AppState;

// Import Vine if needed for future storage change events
// use crate::vine;

// Type aliases for clarity
// 0 = Workspace, 1 = Global
type StorageScope = u32;

// Key-value map for storage
type StorageMap = HashMap<String, Value>;

// --- Helper Functions ---

/// Helper to create a structured error JSON string for RPC error responses.
fn create_error_string(message:String, code:Option<&str>) -> String {
	json!({ "message": message, "code": code.unwrap_or("EUNKNOWN") }).to_string()
}

/// Helper to map Mutex lock poisoning errors to a structured error string.
fn map_lock_error<T>(e:std::sync::PoisonError<MutexGuard<'_, T>>) -> String {
	create_error_string(format!("Failed to acquire lock on storage state: {}", e), Some("ELOCKED"))
}

/// Parses the storage scope (0 for Workspace, 1 for Global) and the string key
/// from the JSON Value parameters received via RPC.
fn get_storage_scope_key(params:&Value) -> Result<(StorageScope, String), String> {
	let scope = params.get("scope").and_then(Value::as_u64).ok_or_else(|| {
		create_error_string(
			"Missing or invalid 'scope' parameter (expected 0 or 1)".to_string(),
			Some("EBADARG"),
		)
	})? as StorageScope;

	if scope != 0 && scope != 1 {
		return Err(create_error_string(
			"Invalid 'scope' value (must be 0 for Workspace or 1 for Global)".to_string(),
			Some("EBADARG"),
		));
	}

	let key = params
		.get("key")
		.and_then(Value::as_str)
		.filter(|s| !s.is_empty())
		.ok_or_else(|| {
			create_error_string(
				"Missing or invalid 'key' parameter (expected non-empty string)".to_string(),
				Some("EBADARG"),
			)
		})?
		.to_string();

	Ok((scope, key))
}

// --- Persistence Helper (Async) ---

/// Asynchronously saves the storage map (JSON) to the specified file path.
/// Creates the parent directory if it doesn't exist.
pub async fn save_storage_to_disk(path:&Path, data:&StorageMap) -> Result<(), String> {
	let json_string =
		serde_json::to_string_pretty(data).map_err(|e| format!("Failed to serialize storage data: {}", e))?;

	if let Some(parent) = path.parent() {
		if !parent.exists() {
			log::info!("[Storage Persistence] Creating storage directory: {}", parent.display());

			fs::create_dir_all(parent)
				.await
				.map_err(|e| format!("Failed to create storage directory {}: {}", parent.display(), e))?;
		}
	} else {
		return Err(format!("Invalid storage path (has no parent): {}", path.display()));
	}

	log::debug!("[Storage Persistence] Writing storage state to {}", path.display());

	let mut file = fs::File::create(path)
		.await
		.map_err(|e| format!("Failed to create/open storage file {}: {}", path.display(), e))?;

	file.write_all(json_string.as_bytes())
		.await
		.map_err(|e| format!("Failed to write storage file {}: {}", path.display(), e))?;

	log::info!("[Storage Persistence] Successfully saved state to {}", path.display());

	Ok(())
}

// --- State Access Helper ---

/// Retrieves the appropriate storage map mutex and file path based on the
/// scope. Made public for potential use during AppState initialization or by
/// effects.
pub fn get_storage_map_mutex_and_path(
	app_state:&AppState,

	scope:StorageScope,
) -> Result<(Arc<StdMutex<StorageMap>>, Option<PathBuf>), String> {
	let mutex = if scope == 1 {
		app_state.global_memento.clone()
	} else {
		app_state.workspace_memento.clone()
	};

	let path_opt = if scope == 1 {
		Some(app_state.global_memento_path.clone())
	} else {
		app_state.workspace_memento_path.clone()
	};

	Ok((mutex, path_opt))
}

// --- RPC Request Handlers (Called by Track dispatcher) ---

/// Handles the `storage_getValue` request from the Cocoon storage shim.
/// Args: `params: { scope: 0 | 1, key: string }`
pub async fn handle_get_value<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let (scope, key) = get_storage_scope_key(params)?;

	let scope_name = if scope == 1 { "Global" } else { "Workspace" };

	// Reduce logging for get requests unless debugging
	log::trace!("[Storage Handler] GetValue scope={}, key='{}'", scope_name, key);

	let app_state = app.state::<AppState>();

	let (storage_mutex, _path_opt) = get_storage_map_mutex_and_path(&app_state, scope)?;

	// TODO: Implement optional load-on-demand if needed
	let storage_guard = storage_mutex
		.lock()
		.map_err(|e| create_handler_error_string(format!("Failed to lock storage state: {}", e), Some("ELOCKED")))?;

	Ok(storage_guard.get(&key).cloned().unwrap_or(Value::Null))
}

/// Handles the `storage_setValue` request from the Cocoon storage shim.
/// Updates the in-memory map and triggers asynchronous persistence to disk.
/// Args: `params: { scope: 0 | 1, key: string, value: any }`
pub async fn handle_set_value<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let (scope, key) = get_storage_scope_key(params)?;

	let value = params
		.get("value")
		.cloned()
		.ok_or_else(|| create_error_string("Missing 'value' parameter".to_string(), Some("EBADARG")))?;

	let scope_name = if scope == 1 { "Global" } else { "Workspace" };

	// Keep log for set operations
	log::info!("[Storage Handler] SetValue scope={}, key='{}'", scope_name, key);

	// Log truncated value: log::debug!("[Storage Handler] Value: '{}...'",

	// value.to_string().chars().take(100).collect::<String>());

	// Validate value is JSON compatible
	if !value.is_null()
		&& !value.is_string()
		&& !value.is_number()
		&& !value.is_boolean()
		&& !value.is_array()
		&& !value.is_object()
	{
		return Err(create_error_string(
			format!("Invalid non-JSON compatible value type received for key '{}'", key),
			Some("EBADARG"),
		));
	}

	let app_state = app.state::<AppState>();

	let (storage_mutex, path_opt) = get_storage_map_mutex_and_path(&app_state, scope)?;

	// Clone data needed for saving *after* the lock is released
	let data_clone_for_save:Option<StorageMap> = {
		let mut storage_guard = storage_mutex.lock().map_err(|e| {
			create_handler_error_string(format!("Failed to lock storage state: {}", e), Some("ELOCKED"))
		})?;

		if value.is_null() {
			log::debug!("[Storage Handler] Deleting key '{}' in scope {}", key, scope_name);

			storage_guard.remove(&key);
		} else {
			storage_guard.insert(key.clone(), value);
		}

		// Clone HashMap for saving
		path_opt.as_ref().map(|_| storage_guard.clone())

		// Lock released here
	};

	// Trigger async save task
	if let (Some(path), Some(data_clone)) = (path_opt, data_clone_for_save) {
		let path_owned = path.clone();

		tokio::spawn(async move {
			log::debug!(
				"[Storage Handler] Persisting {} storage to {}",
				scope_name,
				path_owned.display()
			);

			if let Err(e) = save_storage_to_disk(&path_owned, &data_clone).await {
				log::error!("[Storage Handler] Error persisting storage: {}", e);
			}
		});
	} else if scope != 1 && path_opt.is_none() {
		log::warn!(
			"[Storage Handler] Workspace storage path not set. Cannot persist value for key '{}'.",
			key
		);
	}

	// Return null on success
	Ok(Value::Null)
}
