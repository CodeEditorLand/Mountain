// ---------------------------------------------------------------------------------------------
// Mountain Storage Handlers 
// --------------------------------------------------------------------------------------------
// Implements the backend logic for the Extension Storage API (Memento API).
// These functions are called by the `StorageProvider` trait implementation in
// `environment.rs` to handle storage operations initiated by effects.
//
// Responsibilities:
// - Differentiating between Global and Workspace storage.
// - Accessing and modifying in-memory storage maps in `AppState`.
// - Implementing asynchronous persistence of storage data to disk.
// - Resolving file paths for persistent storage.
//
// Key Interactions:
// - Called by `MountainEnvironment` (implementing `StorageProvider`).
// - Interacts with `AppState` for Memento HashMaps and file paths.
// - Uses `tokio::fs` for asynchronous file writing.
// - Uses `serde_json` for serialization.
// - Returns `CommonError` for effect system compatibility.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{Arc, Mutex as StdMutex, MutexGuard},
};

// CommonError is returned by these handlers for the effect system.
use Land_Common::errors::CommonError;
use log::{debug, error, info, trace, warn};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use tokio::{fs, io::AsyncWriteExt}; // Tokio fs for async operations

use crate::app_state::AppState;

// --- Type Aliases ---

/// Type alias for storage scope identifier.
/// `0` represents Workspace-scoped storage.
/// `1` represents Global-scoped storage.
pub type StorageScope = u32; // 0 = Workspace, 1 = Global

/// Type alias for the in-memory representation of a Memento storage map.
pub type StorageMap = HashMap<String, Value>;

// --- Helper Functions ---

/// Asynchronously saves the provided storage map (as JSON) to the specified
/// file path. Creates parent directories if they don't exist.
pub async fn save_storage_map_to_disk(path:&Path, data:&StorageMap) -> Result<(), String> {
	let json_string = serde_json::to_string_pretty(data).map_err(|e| {
		format!(
			"[Storage Persistence] Failed to serialize storage data for path '{}': {}",
			path.display(),
			e
		)
	})?;

	if let Some(parent_dir) = path.parent() {
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
		return Err(format!(
			"[Storage Persistence] Invalid storage path (no parent): {}",
			path.display()
		));
	}

	debug!(
		"[Storage Persistence] Writing storage ({} keys) to: {}",
		data.len(),
		path.display()
	);
	let mut file = fs::File::create(path).await.map_err(|e| {
		format!(
			"[Storage Persistence] Failed to create/open storage file '{}': {}",
			path.display(),
			e
		)
	})?;
	file.write_all(json_string.as_bytes()).await.map_err(|e| {
		format!(
			"[Storage Persistence] Failed to write storage data to '{}': {}",
			path.display(),
			e
		)
	})?;
	info!("[Storage Persistence] Successfully saved storage to {}", path.display());
	Ok(())
}

/// Retrieves the appropriate storage map `Arc<StdMutex<StorageMap>>` and its
/// persistence file `PathBuf` from `AppState` based on the given `scope`.
pub fn get_storage_map_and_path_from_appstate(
	app_state:&AppState,
	scope:StorageScope, // 0 for Workspace, 1 for Global
) -> Result<(Arc<StdMutex<StorageMap>>, Option<PathBuf>), CommonError> {
	let memento_map_mutex = if scope == 1 {
		// Global
		app_state.global_memento.clone()
	} else {
		// Workspace (scope == 0)
		app_state.workspace_memento.clone()
	};

	let memento_file_path_opt = if scope == 1 {
		// Global
		Some(app_state.global_memento_path.clone())
	} else {
		// Workspace
		match app_state.workspace_memento_path.lock() {
			Ok(guard) => guard.clone(),
			Err(e) => return Err(CommonError::StateLock(format!("Failed to lock workspace_memento_path: {}", e))),
		}
	};
	Ok((memento_map_mutex, memento_file_path_opt))
}

// --- Handler Logic  ---

/// Implements the logic for `StorageProvider::get_storage_value`.
pub async fn handle_get_storage_value_effect_logic<R:Runtime>(
	app_handle:AppHandle<R>,
	is_global_scope:bool,
	key:&str,
) -> Result<Option<Value>, CommonError> {
	let scope_name = if is_global_scope { "Global" } else { "Workspace" };
	trace!("[Storage Handler Effect] GetValue: scope={}, key='{}'", scope_name, key);

	let app_state = app_handle.state::<AppState>();
	let (storage_map_mutex, _) =
		get_storage_map_and_path_from_appstate(&app_state, if is_global_scope { 1 } else { 0 })?;

	let storage_map_guard = storage_map_mutex
		.lock()
		.map_err(|e| CommonError::StateLock(format!("Storage lock error for get_storage_value: {}", e)))?;

	Ok(storage_map_guard.get(key).cloned())
}

/// Implements the logic for `StorageProvider::update_storage_value`.
pub async fn handle_set_storage_value_effect_logic<R:Runtime>(
	app_handle:AppHandle<R>,
	is_global_scope:bool,
	key:String,                 // Owned
	value_to_set:Option<Value>, // Option<Value> directly
) -> Result<(), CommonError> {
	let scope_name_str = if is_global_scope { "Global" } else { "Workspace" };
	info!(
		"[Storage Handler Effect] SetValue: scope={}, key='{}', value_is_some={}",
		scope_name_str,
		key,
		value_to_set.is_some()
	);
	trace!("[Storage Handler Effect] Value for key '{}': {:?}", key, value_to_set);

	let app_state = app_handle.state::<AppState>();
	let (storage_map_mutex, storage_file_path_opt) =
		get_storage_map_and_path_from_appstate(&app_state, if is_global_scope { 1 } else { 0 })?;

	let data_clone_for_async_save:Option<StorageMap> = {
		let mut storage_map_guard = storage_map_mutex
			.lock()
			.map_err(|e| CommonError::StateLock(format!("Storage lock error for set_storage_value: {}", e)))?;
		if let Some(val) = value_to_set {
			debug!(
				"[Storage Handler Effect] Inserting/Updating key '{}' in {} Memento.",
				key, scope_name_str
			);
			storage_map_guard.insert(key.clone(), val);
		} else {
			debug!(
				"[Storage Handler Effect] Deleting key '{}' from {} Memento.",
				key, scope_name_str
			);
			storage_map_guard.remove(&key);
		}
		// Clone the entire HashMap for saving only if a persistence path is available
		storage_file_path_opt.as_ref().map(|_| storage_map_guard.clone())
	};

	if let (Some(path_to_save_to), Some(data_to_persist)) = (storage_file_path_opt, data_clone_for_async_save) {
		let path_owned_for_task = path_to_save_to.clone(); // Clone for async task
		let scope_name_for_task = scope_name_str.to_string(); // Clone for async task
		tokio::spawn(async move {
			// Spawn persistence task
			debug!(
				"[Storage Handler Effect Task] Persisting {} storage ({} keys) to: {}",
				scope_name_for_task,
				data_to_persist.len(),
				path_owned_for_task.display()
			);
			if let Err(e_str) = save_storage_map_to_disk(&path_owned_for_task, &data_to_persist).await {
				error!(
					"[Storage Handler Effect Task] Error persisting {} storage to '{}': {}",
					scope_name_for_task,
					path_owned_for_task.display(),
					e_str
				);
				// TODO: Consider retry mechanism or user notification for
				// persistent storage failures.
			}
		});
	} else if !is_global_scope && storage_file_path_opt.is_none() {
		warn!(
			"[Storage Handler Effect] Workspace storage path not set. Cannot persist key '{}'. Change is in-memory \
			 only.",
			key
		);
	}
	Ok(())
}

// NEW:
// // Example signature in Handler/storage.rs
// pub async fn handle_get_storage_value_effect_logic<R: tauri::Runtime>(
//     app_handle: tauri::AppHandle<R>,
//     is_global_scope: bool,
//     key: &str,
// ) -> Result<Option<Value>, CommonError> {
//     // ... implementation ...
//     todo!()
// }

// pub async fn handle_set_storage_value_effect_logic<R: tauri::Runtime>(
//     app_handle: tauri::AppHandle<R>,
//     is_global_scope: bool,
//     key: String,
//     value_to_set: Option<Value>,
// ) -> Result<(), CommonError> {
//     // ... implementation ...
//     todo!()
// }
