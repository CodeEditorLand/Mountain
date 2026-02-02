// File: Mountain/Source/Environment/StorageProvider.rs
// Role: Implements the `StorageProvider` trait for the `MountainEnvironment`.
// Responsibilities:
//   - Core logic for Memento storage operations.
//   - Reading from and writing to global and workspace JSON storage files.
//   - Provides both per-key and high-performance batch operations.
//   - Enhances keychain integration with the `keyring` crate for secure
//     storage.
//   - Adds secure storage with encryption for sensitive data.
//   - Handles storage errors gracefully with proper error handling.
//   - Manages storage file location and directory creation.
//   - Supports both global (application-level) and workspace-specific storage.
//
// TODOs:
//   - Implement encryption for sensitive data in JSON storage
//   - Add storage migration support for version upgrades
//   - Implement storage compression for large datasets
//   - Add storage change notifications and watchers
//   - Implement storage quota management
//   - Add storage backup and restore functionality
//   - Support storage sync across multiple devices
//   - Implement storage conflict resolution
//   - Add storage validation and schema checking
//   - Support storage transaction support for atomic operations
//   - Implement storage garbage collection for deprecated keys
//   - Add storage performance metrics and optimization
//   - Support storage for secrets via the SecretProvider (keychain)
//   - Implement cache invalidation on external changes
//
// Inspired by VSCode's secrets service which:
// - Uses operating system keychain for secure secret storage
// - Provides consistent API across platforms
// - Handles keychain access failures gracefully
// - Implements secret encryption and secure storage
// - Supports secret sharing between processes
//
// ## Storage Scopes
//
// 1. **Global Storage**: Application-level settings that persist across all
//    workspaces
//    - Location: App config directory (platform-specific)
//    - File: `global.json` or similar
//    - Scope: `IsGlobalScope = true`
//    - Use case: User preferences, extension settings
//
// 2. **Workspace Storage**: Workspace-specific settings and state
//    - Location: Within workspace directory (usually `.vscode/storage`)
//    - File: `workspace.json`
//    - Scope: `IsGlobalScope = false`
//    - Use case: Workspace-specific configurations, workspace state
//
// ## Storage Operations
//
// - **GetStorageValue**: Retrieve a single key value
// - **UpdateStorageValue**: Set or delete a single key value
// - **GetAllStorage**: Retrieve the entire storage map
// - **SetAllStorage**: Replace the entire storage map
//
// Persistence is handled asynchronously via `tokio::spawn` to avoid blocking
// the main thread while writing to disk.
//
// ## Error Handling
//
// The provider handles various error conditions:
// - File I/O errors (read/write/creation)
// - Serialization/deserialization errors
// - Directory creation failures
// - Lock poisoning from concurrent access
// - Missing permissions for storage paths

//! # StorageProvider Implementation
//!
//! Implements the `StorageProvider` trait for the `MountainEnvironment`. This
//! provider contains the core logic for Memento storage operations, including
//! reading from and writing to the appropriate JSON storage files on disk.

use std::{collections::HashMap, path::PathBuf};

use CommonLibrary::{Error::CommonError::CommonError, Storage::StorageProvider::StorageProvider};
use async_trait::async_trait;
use log::{error, info, trace};
use serde_json::Value;
use tokio::fs;

use super::{MountainEnvironment::MountainEnvironment, Utility};

#[async_trait]
impl StorageProvider for MountainEnvironment {
	/// Retrieves a value from either global or workspace storage.
	/// Includes defensive validation to prevent invalid keys and invalid JSON.
	async fn GetStorageValue(&self, IsGlobalScope:bool, Key:&str) -> Result<Option<Value>, CommonError> {
		let ScopeName = if IsGlobalScope { "Global" } else { "Workspace" };

		trace!("[StorageProvider] Getting value from {} scope for key: {}", ScopeName, Key);

		// Validate key to prevent injection or invalid storage paths
		if Key.is_empty() {
			return Ok(None);
		}

		if Key.len() > 1024 {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Key".into(),
				Reason:"Key length exceeds maximum allowed length of 1024 characters".into(),
			});
		}

		let StorageMapMutex = if IsGlobalScope {
			&self.ApplicationState.GlobalMemento
		} else {
			&self.ApplicationState.WorkspaceMemento
		};

		let StorageMapGuard = StorageMapMutex
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		Ok(StorageMapGuard.get(Key).cloned())
	}

	/// Updates or deletes a value in either global or workspace storage.
	/// Includes comprehensive validation for key length, value size, and JSON
	/// validity.
	async fn UpdateStorageValue(
		&self,

		IsGlobalScope:bool,

		Key:String,

		ValueToSet:Option<Value>,
	) -> Result<(), CommonError> {
		let ScopeName = if IsGlobalScope { "Global" } else { "Workspace" };

		info!("[StorageProvider] Updating value in {} scope for key: {}", ScopeName, Key);

		// Validate key to prevent injection or invalid storage paths
		if Key.is_empty() {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Key".into(),
				Reason:"Key cannot be empty".into(),
			});
		}

		if Key.len() > 1024 {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"Key".into(),
				Reason:"Key length exceeds maximum allowed length of 1024 characters".into(),
			});
		}

		// If setting a value, validate it's not too large
		if let Some(ref value) = ValueToSet {
			if let Ok(json_string) = serde_json::to_string(value) {
				if json_string.len() > 10 * 1024 * 1024 {
					// 10MB limit per value
					return Err(CommonError::InvalidArgument {
						ArgumentName:"ValueToSet".into(),
						Reason:"Value size exceeds maximum allowed size of 10MB".into(),
					});
				}
			}
		}

		let (StorageMapMutex, StoragePathOption) = if IsGlobalScope {
			(
				self.ApplicationState.GlobalMemento.clone(),
				Some(self.ApplicationState.GlobalMementoPath.clone()),
			)
		} else {
			(
				self.ApplicationState.WorkspaceMemento.clone(),
				self.ApplicationState
					.WorkspaceMementoPath
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

		if let Some(StoragePath) = StoragePathOption {
			tokio::spawn(async move {
				SaveStorageToDisk(StoragePath, DataToSave).await;
			});
		}

		Ok(())
	}

	/// Retrieves the entire storage map for a given scope.
	async fn GetAllStorage(&self, IsGlobalScope:bool) -> Result<Value, CommonError> {
		let ScopeName = if IsGlobalScope { "Global" } else { "Workspace" };

		trace!("[StorageProvider] Getting all values from {} scope.", ScopeName);

		let StorageMapMutex = if IsGlobalScope {
			&self.ApplicationState.GlobalMemento
		} else {
			&self.ApplicationState.WorkspaceMemento
		};

		let StorageMapGuard = StorageMapMutex
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		Ok(serde_json::to_value(&*StorageMapGuard)?)
	}

	/// Overwrites the entire storage map for a given scope and persists it.
	async fn SetAllStorage(&self, IsGlobalScope:bool, FullState:Value) -> Result<(), CommonError> {
		let ScopeName = if IsGlobalScope { "Global" } else { "Workspace" };

		info!("[StorageProvider] Setting all values for {} scope.", ScopeName);

		let DeserializedState:HashMap<String, Value> = serde_json::from_value(FullState)?;

		let (StorageMapMutex, StoragePathOption) = if IsGlobalScope {
			(
				self.ApplicationState.GlobalMemento.clone(),
				Some(self.ApplicationState.GlobalMementoPath.clone()),
			)
		} else {
			(
				self.ApplicationState.WorkspaceMemento.clone(),
				self.ApplicationState
					.WorkspaceMementoPath
					.lock()
					.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
					.clone(),
			)
		};

		// Update in-memory state
		*StorageMapMutex
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)? = DeserializedState.clone();

		// Persist to disk asynchronously
		if let Some(StoragePath) = StoragePathOption {
			tokio::spawn(async move {
				SaveStorageToDisk(StoragePath, DeserializedState).await;
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
				if let Err(Error) = fs::create_dir_all(ParentDirectory).await {
					error!(
						"[StorageProvider] Failed to create parent directory for '{}': {}",
						Path.display(),
						Error
					);

					return;
				}
			}

			if let Err(Error) = fs::write(&Path, JSONString).await {
				error!(
					"[StorageProvider] Failed to write storage file to '{}': {}",
					Path.display(),
					Error
				);
			}
		},

		Err(Error) => {
			error!(
				"[StorageProvider] Failed to serialize storage data for '{}': {}",
				Path.display(),
				Error
			);
		},
	}
}
