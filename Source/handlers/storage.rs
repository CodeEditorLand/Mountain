// ---------------------------------------------------------------------------------------------
// Mountain Storage Handlers (handlers/storage.rs)
// --------------------------------------------------------------------------------------------
// Implements the backend logic for the Extension Storage API (Memento),

// handling persistent key-value storage for extensions running in sidecars
// (Cocoon).
//
// Responsibilities:
// - Handling `$getValue` and `$setValue` RPC calls (via effects in Track).
// - Differentiating between Global and Workspace storage scopes.
// - Accessing in-memory storage maps in `AppState`.
// - Implementing persistence: loading (in app_state.rs) and saving to disk.
// - Resolving file paths for persistence.
//
// Key Interactions:
// - Called by effects created in `track.rs` or by `environment.rs`.
// - Interacts with `AppState` for Memento HashMaps and storage paths.
// - Uses `tokio::fs` for asynchronous file writing.
// - Uses `serde_json` for serialization/deserialization.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{Arc, Mutex as StdMutex, MutexGuard},
};

use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
// Removed State as it's not directly used in RPC handlers
use tauri::{AppHandle, Manager, Runtime};
use tokio::{fs, io::AsyncWriteExt};

use crate::app_state::AppState;
// Use shared error utilities
use crate::handlers::error_utils;

// Not directly returned by these RPC handlers
// use Land_Common::errors::CommonError;

// Type aliases
// 0 = Workspace, 1 = Global
// Made pub for use in environment.rs
pub type StorageScope = u32;

// Exporting for environment.rs
pub type StorageMap = HashMap<String, Value>;

// --- Helper Functions ---

/// Helper to map Mutex lock poisoning errors for storage state.
fn map_storage_lock_error_to_str<T>(e:std::sync::PoisonError<MutexGuard<'_, T>>) -> String {
	let msg = format!("[Storage Handler LockErr] Failed to acquire lock on storage state: {}", e);

	// Log the specific error
	error!("{}", msg);

	error_utils::rpc_error_string(msg, Some("ELOCKED_STORAGE"))
}

/// Parses the storage scope (0 for Workspace, 1 for Global) and the string key
/// from the JSON Value parameters received via RPC.
fn get_storage_scope_key(params:&Value, operation_name:&str) -> Result<(StorageScope, String), String> {
	let scope = params.get("scope").and_then(Value::as_u64).ok_or_else(|| {
		error_utils::rpc_param_error_string(operation_name, "scope", "0 (Workspace) or 1 (Global)", Some(0))
	})? as StorageScope;

	if scope != 0 && scope != 1 {
		return Err(error_utils::rpc_error_string(
			"Invalid 'scope' value (must be 0 for Workspace or 1 for Global)".to_string(),
			Some("EBADARG_SCOPE"),
		));
	}

	let key = params
		.get("key")
		.and_then(Value::as_str)
		.filter(|s| !s.is_empty())
		.ok_or_else(|| error_utils::rpc_param_error_string(operation_name, "key", "non-empty string", Some(0)))?
		.to_string();

	Ok((scope, key))
}

// --- Persistence Helper (Async) ---

/// Asynchronously saves the storage map (JSON) to the specified file path.
/// Creates the parent directory if it doesn't exist.
pub async fn save_storage_to_disk(path:&Path, data:&StorageMap) -> Result<(), String> {
	let json_string = serde_json::to_string_pretty(data).map_err(|e| {
		format!(
			"[Storage Persistence] Failed to serialize storage data for {}: {}",
			path.display(),
			e
		)
	})?;

	if let Some(parent) = path.parent() {
		// Use tokio::fs for async check
		if !tokio::fs::try_exists(parent).await.unwrap_or(false) {
			info!("[Storage Persistence] Creating storage directory: {}", parent.display());

			// Use tokio::fs
			fs::create_dir_all(parent).await.map_err(|e| {
				format!(
					"[Storage Persistence] Failed to create storage directory {}: {}",
					parent.display(),
					e
				)
			})?;
		}
	} else {
		return Err(format!(
			"[Storage Persistence] Invalid storage path (has no parent): {}",
			path.display()
		));
	}

	debug!("[Storage Persistence] Writing storage state to {}", path.display());

	// Use tokio::fs
	let mut file = fs::File::create(path).await.map_err(|e| {
		format!(
			"[Storage Persistence] Failed to create/open storage file {}: {}",
			path.display(),
			e
		)
	})?;

	file.write_all(json_string.as_bytes())
		.await
		.map_err(|e| format!("[Storage Persistence] Failed to write storage file {}: {}", path.display(), e))?;

	// Keep: Confirmation log
	info!("[Storage Persistence] Successfully saved state to {}", path.display());

	Ok(())
}

// --- State Access Helper ---

/// Retrieves the appropriate storage map mutex and file path based on scope.
/// Made public for use by storage effects in `environment.rs` or `app_state.rs`
/// init.
pub fn get_storage_map_and_path(
	app_state:&AppState,

	scope:StorageScope,
) -> Result<(Arc<StdMutex<StorageMap>>, Option<PathBuf>), String> {
	// Global
	let mutex = if scope == 1 {
		app_state.global_memento.clone()
		// Workspace (scope == 0)
	} else {
		app_state.workspace_memento.clone()
	};

	let path_opt = if scope == 1 {
		Some(app_state.global_memento_path.clone())
	} else {
		// Workspace path itself is Arc<StdMutex<Option<PathBuf>>>

		match app_state.workspace_memento_path.lock() {
			// Clone the Option<PathBuf> from within the guard
			Ok(guard) => guard.clone(),

			Err(e) => return Err(map_storage_lock_error_to_str(e)),
		}
	};

	Ok((mutex, path_opt))
}

// --- RPC Request Handlers (Called by effects or direct RPC dispatcher) ---

/// Handles the `storage_getValue` request from the Cocoon storage shim.
/// Args: `params: { scope: 0 | 1, key: string }`
pub async fn handle_get_value<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let (scope, key) = get_storage_scope_key(params, "storage_getValue")?;

	let scope_name = if scope == 1 { "Global" } else { "Workspace" };

	// Reduce logging for get requests unless debugging
	trace!("[Storage Handler] GetValue scope={}, key='{}'", scope_name, key);

	let app_state = app.state::<AppState>();

	let (storage_mutex, _path_opt) = get_storage_map_and_path(&app_state, scope)?;

	let storage_guard = storage_mutex.lock().map_err(map_storage_lock_error_to_str)?;

	Ok(storage_guard.get(&key).cloned().unwrap_or(Value::Null))
}

/// Handles the `storage_setValue` request from the Cocoon storage shim.
/// Updates the in-memory map and triggers asynchronous persistence to disk.
/// Args: `params: { scope: 0 | 1, key: string, value: any }`
pub async fn handle_set_value<R:Runtime>(app:AppHandle<R>, params:Value) -> Result<Value, String> {
	let (scope, key) = get_storage_scope_key(params, "storage_setValue")?;

	// value can be null to delete
	let value_to_set = params.get("value").cloned().ok_or_else(|| {
		error_utils::rpc_param_error_string("storage_setValue", "value", "any JSON value or null", Some(0))
	})?;

	let scope_name = if scope == 1 { "Global" } else { "Workspace" };

	// Keep log for set operations
	info!("[Storage Handler] SetValue scope={}, key='{}'", scope_name, key);

	trace!(
		"[Storage Handler] Value for '{}': {}...",
		key,
		value_to_set.to_string().chars().take(100).collect::<String>()
	);

	let app_state = app.state::<AppState>();

	let (storage_mutex, path_opt) = get_storage_map_and_path(&app_state, scope)?;

	// Clone data needed for saving *after* the lock is released
	let data_clone_for_save:Option<StorageMap> = {
		let mut storage_guard = storage_mutex.lock().map_err(map_storage_lock_error_to_str)?;

		if value_to_set.is_null() {
			debug!("[Storage Handler] Deleting key '{}' in scope {}", key, scope_name);

			storage_guard.remove(&key);
		} else {
			storage_guard.insert(key.clone(), value_to_set);
		}

		// Clone HashMap for saving only if a path is available for this scope
		path_opt.as_ref().map(|_| storage_guard.clone())
	};

	// Trigger async save task
	if let (Some(path), Some(data_clone)) = (path_opt, data_clone_for_save) {
		// Clone for the async task
		let path_owned = path.clone();

		tokio::spawn(async move {
			debug!(
				"[Storage Handler] Persisting {} storage to {}",
				scope_name,
				path_owned.display()
			);

			if let Err(e_str) = save_storage_to_disk(&path_owned, &data_clone).await {
				error!(
					"[Storage Handler] Error persisting {} storage to {}: {}",
					scope_name,
					path_owned.display(),
					e_str
				);
			}
		});

		// If workspace scope but no path
	} else if scope != 1 && path_opt.is_none() {
		warn!(
			"[Storage Handler] Workspace storage path not set. Cannot persist value for key '{}'. Change will only be \
			 in memory.",
			key
		);
	}

	// Return null on success
	Ok(Value::Null)
}
