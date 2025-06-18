// @module StorageLogic
// @description Contains the core logic for Memento storage operations,
// including reading from and writing to the appropriate JSON storage files on
// disk.

use std::{collections::HashMap, path::PathBuf};

use Common::error::CommonError;
use log::{error, info, trace};
use serde_json::Value;
use tauri::{AppHandle, Manager, Runtime};
use tokio::fs;

use crate::{ApplicationState::ApplicationState::ApplicationState, Environment::Utility};

// An internal helper function to asynchronously write the storage map to a
// file.
async fn save_storage_to_disk(path:PathBuf, data:HashMap<String, Value>) {
	trace!("[StorageLogic] Persisting storage to disk: {}", path.display());
	match serde_json::to_string_pretty(&data) {
		Ok(json_string) => {
			if let Some(parent_dir) = path.parent() {
				if let Err(e) = fs::create_dir_all(parent_dir).await {
					error!(
						"[StorageLogic] Failed to create parent directory for storage file '{}': {}",
						path.display(),
						e
					);
					return;
				}
			}
			if let Err(e) = fs::write(&path, json_string).await {
				error!("[StorageLogic] Failed to write storage file to '{}': {}", path.display(), e);
			}
		},
		Err(e) => {
			error!(
				"[StorageLogic] Failed to serialize storage data for '{}': {}",
				path.display(),
				e
			);
		},
	}
}

// Logic to retrieve a value from either global or workspace storage.
pub async fn GetStorageValueLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	is_global_scope:bool,
	key:&str,
) -> Result<Option<Value>, CommonError> {
	let scope_name = if is_global_scope { "Global" } else { "Workspace" };
	trace!("[StorageLogic] Getting value from {} scope for key: {}", scope_name, key);
	let app_state = app_handle.state::<ApplicationState>();

	let storage_map_mutex = if is_global_scope {
		&app_state.GlobalMemento
	} else {
		&app_state.WorkspaceMemento
	};

	let storage_map_guard = storage_map_mutex.lock().map_err(Utility::MapAppStateLockErrorToCommonError)?;
	Ok(storage_map_guard.get(key).cloned())
}

// Logic to update or delete a value in either global or workspace storage.
pub async fn UpdateStorageValueLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	is_global_scope:bool,
	key:String,
	value_to_set:Option<Value>,
) -> Result<(), CommonError> {
	let scope_name = if is_global_scope { "Global" } else { "Workspace" };
	info!("[StorageLogic] Updating value in {} scope for key: {}", scope_name, key);
	let app_state = app_handle.state::<ApplicationState>();

	let (storage_map_mutex, storage_path_opt) = if is_global_scope {
		(app_state.GlobalMemento.clone(), Some(app_state.GlobalMementoPath.clone()))
	} else {
		(
			app_state.WorkspaceMemento.clone(),
			app_state
				.WorkspaceMementoPath
				.lock()
				.map_err(Utility::MapAppStateLockErrorToCommonError)?
				.clone(),
		)
	};

	// Perform the in-memory update.
	let data_to_save = {
		let mut storage_map_guard = storage_map_mutex.lock().map_err(Utility::MapAppStateLockErrorToCommonError)?;
		if let Some(value) = value_to_set {
			storage_map_guard.insert(key, value);
		} else {
			storage_map_guard.remove(&key);
		}
		storage_map_guard.clone()
	};

	// If a path is configured, spawn a background task to persist the changes.
	// NOTE: This writes the entire file on every change. A more advanced
	// implementation would use a debounced writer to batch multiple changes
	// together.
	if let Some(storage_path) = storage_path_opt {
		tokio::spawn(async move {
			save_storage_to_disk(storage_path, data_to_save).await;
		});
	}

	Ok(())
}
