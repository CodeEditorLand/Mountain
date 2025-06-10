use std::{collections::HashMap, path::PathBuf};

use Common::error::CommonError;
use log::{error, info, trace};
use serde_json::Value;
use tauri::{ApplicationHandle, Manager, RunTime};
use tokio::fs;

// @module StorageLogic
// @description Contains the core logic for Memento storage operations,
// including reading from and writing to the appropriate JSON storage files on
// disk.
use crate::{ApplicationState::ApplicationState::ApplicationState, environment::Utils};

// An internal helper function to asynchronously write the storage map to a
// file.
async fn SaveStorageToDisk(Path:PathBuf, Data:HashMap<String, Value>) {
	trace!("[StorageLogic] Persisting storage to disk: {}", Path.display());
	match serde_json::to_string_pretty(&Data) {
		Ok(JsonString) => {
			if let Some(ParentDir) = Path.parent() {
				if let Err(e) = fs::create_dir_all(ParentDir).await {
					error!(
						"[StorageLogic] Failed to create parent directory for storage file '{}': {}",
						Path.display(),
						e
					);
					return;
				}
			}
			if let Err(e) = fs::write(&Path, JsonString).await {
				error!("[StorageLogic] Failed to write storage file to '{}': {}", Path.display(), e);
			}
		},
		Err(e) => {
			error!(
				"[StorageLogic] Failed to serialize storage data for '{}': {}",
				Path.display(),
				e
			);
		},
	}
}

// Logic to retrieve a value from either global or workspace storage.
pub async fn GetStorageValueLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	IsGlobalScope:bool,
	Key:&str,
) -> Result<Option<Value>, CommonError> {
	let ScopeName = if IsGlobalScope { "Global" } else { "Workspace" };
	trace!("[StorageLogic] Getting value from {} scope for key: {}", ScopeName, Key);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();

	let StorageMapMutex = if IsGlobalScope {
		&AppStateInstance.GlobalMemento
	} else {
		&AppStateInstance.WorkspaceMemento
	};

	let StorageMapGuard = StorageMapMutex.lock().map_err(Utils::MapAppStateLockErrorToCommonError)?;
	Ok(StorageMapGuard.get(Key).cloned())
}

// Logic to update or delete a value in either global or workspace storage.
pub async fn UpdateStorageValueLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	IsGlobalScope:bool,
	Key:String,
	ValueToSet:Option<Value>,
) -> Result<(), CommonError> {
	let ScopeName = if IsGlobalScope { "Global" } else { "Workspace" };
	info!("[StorageLogic] Updating value in {} scope for key: {}", ScopeName, Key);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();

	let (StorageMapMutex, StoragePathOpt) = if IsGlobalScope {
		(
			AppStateInstance.GlobalMemento.clone(),
			Some(AppStateInstance.GlobalMementoPath.clone()),
		)
	} else {
		(
			AppStateInstance.WorkspaceMemento.clone(),
			AppStateInstance
				.WorkspaceMementoPath
				.lock()
				.map_err(Utils::MapAppStateLockErrorToCommonError)?
				.clone(),
		)
	};

	// Perform the in-memory update.
	let DataToSave = {
		let mut StorageMapGuard = StorageMapMutex.lock().map_err(Utils::MapAppStateLockErrorToCommonError)?;
		if let Some(Value) = ValueToSet {
			StorageMapGuard.insert(Key, Value);
		} else {
			StorageMapGuard.remove(&Key);
		}
		StorageMapGuard.clone()
	};

	// If a path is configured, spawn a background task to persist the changes.
	// NOTE: This writes the entire file on every change. A more advanced
	// implementation would use a debounced writer to batch multiple changes
	// together.
	if let Some(StoragePath) = StoragePathOpt {
		tokio::spawn(async move {
			SaveStorageToDisk(StoragePath, DataToSave).await;
		});
	}

	Ok(())
}
