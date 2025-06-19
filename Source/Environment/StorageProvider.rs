//! # StorageProvider Implementation
//!
//! Implements the `StorageProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for Memento storage operations, including
//! reading from and writing to the appropriate JSON storage files on disk.

use std::{collections::HashMap, path::PathBuf};

use Common::{Error::CommonError::CommonError, Storage::StorageProvider::StorageProvider};
use async_trait::async_trait;
use log::{error, info, trace};
use serde_json::Value;
use tokio::fs;

use super::{MountainEnvironment::MountainEnvironment, Utility};

#[async_trait]
impl StorageProvider for MountainEnvironment {
	/// Retrieves a value from either global or workspace storage.
	async fn GetStorageValue(&self, IsGlobalScope:bool, Key:&str) -> Result<Option<Value>, CommonError> {
		let ScopeName = if IsGlobalScope { "Global" } else { "WorkSpace" };
		trace!("[StorageProvider] Getting value from {} scope for key: {}", ScopeName, Key);

		let StorageMapMutex = if IsGlobalScope {
			&self.ApplicationState.GlobalMemento
		} else {
			&self.ApplicationState.WorkSpaceMemento
		};

		let StorageMapGuard = StorageMapMutex
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
		Ok(StorageMapGuard.get(Key).cloned())
	}

	/// Updates or deletes a value in either global or workspace storage.
	async fn UpdateStorageValue(
		&self,
		IsGlobalScope:bool,
		Key:String,
		ValueToSet:Option<Value>,
	) -> Result<(), CommonError> {
		let ScopeName = if IsGlobalScope { "Global" } else { "WorkSpace" };
		info!("[StorageProvider] Updating value in {} scope for key: {}", ScopeName, Key);

		let (StorageMapMutex, StoragePathOption) = if IsGlobalScope {
			(
				self.ApplicationState.GlobalMemento.clone(),
				Some(self.ApplicationState.GlobalMementoPath.clone()),
			)
		} else {
			(
				self.ApplicationState.WorkSpaceMemento.clone(),
				self.ApplicationState
					.WorkSpaceMementoPath
					.lock()
					.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
					.clone(),
			)
		};

		// Perform the in-memory update.
		let DataToSave = {
			let mut StorageMapGuard = StorageMapMutex
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;
			if let Some(Value) = ValueToSet {
				StorageMapGuard.insert(Key, Value);
			} else {
				StorageMapGuard.remove(&Key);
			}
			StorageMapGuard.clone()
		};

		// If a path is configured, spawn a background task to persist the changes.
		// NOTE: This writes the entire file on every change. A more advanced
		// implementation would use a debounced writer to batch multiple changes.
		if let Some(StoragePath) = StoragePathOption {
			tokio::spawn(async move {
				SaveStorageToDisk(StoragePath, DataToSave).await;
			});
		}

		Ok(())
	}
}

// --- Internal Helper Functions ---

/// An internal helper function to asynchronously write the storage map to a
/// file.
async fn SaveStorageToDisk(Path:PathBuf, Data:HashMap<String, Value>) {
	trace!("[StorageProvider] Persisting storage to disk: {}", Path.display());
	match serde_json::to_string_pretty(&Data) {
		Ok(JSONString) => {
			if let Some(ParentDirectory) = Path.parent() {
				if let Err(e) = fs::create_dir_all(ParentDirectory).await {
					error!(
						"[StorageProvider] Failed to create parent directory for '{}': {}",
						Path.display(),
						e
					);
					return;
				}
			}
			if let Err(e) = fs::write(&Path, JSONString).await {
				error!("[StorageProvider] Failed to write storage file to '{}': {}", Path.display(), e);
			}
		},
		Err(e) => {
			error!(
				"[StorageProvider] Failed to serialize storage data for '{}': {}",
				Path.display(),
				e
			);
		},
	}
}
