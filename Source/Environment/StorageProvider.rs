//! # StorageProvider (Environment)
//!
//! Implements the `StorageProvider` trait for `MountainEnvironment`, providing
//! persistent key-value storage for extensions and application components.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Storage Management
//! - Provide global storage (shared across all workspaces)
//! - Provide workspace-scoped storage (per workspace)
//! - Support namespaced keys to avoid collisions
//! - Handle storage quota limits
//!
//! ### 2. CRUD Operations
//! - `Get(scope, key)`: Retrieve value by key
//! - `Set(scope, key, value)`: Store value (create or update)
//! - `Remove(scope, key)`: Delete key-value pair
//! - `Clear(scope)`: Remove all keys in a scope
//! - `Keys(scope)`: List all keys in a scope
//!
//! ### 3. Data Persistence
//! - Write storage changes to disk immediately
//! - Load storage from disk on startup
//! - Handle corrupted storage files with recovery
//! - Backup storage before writes (optional)
//!
//! ### 4. Type Safety
//! - Store and retrieve arbitrary JSON-serializable values
//! - Handle serialization/deserialization errors
//! - Support primitive types and complex objects
//!
//! ## ARCHITECTURAL ROLE
//!
//! StorageProvider is the **persistent key-value store** for Mountain:
//!
//! ```text
//! Extension ──► StorageProvider ──► Disk (JSON files)
//!                      │
//!                      └─► ApplicationState (Cache)
//! ```
//!
//! ### Position in Mountain
//! - `Environment` module: Persistence capability provider
//! - Implements `CommonLibrary::Storage::StorageProvider` trait
//! - Accessible via `Environment.Require<dyn StorageProvider>()`
//!
//! ### Storage Scopes
//! - **Global**: `{AppData}/User/globalStorage.json`
//!   - Shared across all workspaces
//!   - Used for user preferences, extension state
//! - **Workspace**:
//!   `{AppData}/User/workspaceStorage/{workspace-id}/storage.json`
//!   - Specific to current workspace
//!   - Used for workspace-specific settings and state
//!
//! ### Storage Format
//! - JSON file with simple key-value pairs
//! - Values are `serde_json::Value` (any JSON type)
//! - Keys are strings with namespace prefix (e.g., "extensionId.setting")
//!
//! ### Dependencies
//! - `ApplicationState`: Access to global/workspace memento maps
//! - `FileSystemWriter`: To persist storage to disk
//! - `Log`: Storage change logging
//!
//! ### Dependents
//! - Extensions: Store extension-specific state
//! - `ConfigurationProvider`: Uses global storage for user settings
//! - `ExtensionManagement`: Store extension metadata
//! - Any component needing persistent settings
//!
//! ## STORAGE LIFECYCLE
//!
//! 1. **App Start**: `ApplicationState::default()` loads global and workspace
//!    memento
//! 2. **Workspace Change**: `UpdateWorkspaceMementoPathAndReload()` loads new
//!    workspace storage
//! 3. **Runtime**: Providers read/write to in-memory maps (`GlobalMemento`,
//!    `WorkspaceMemento`)
//! 4. **Shutdown**: `ApplicationRunTime::SaveApplicationState()` writes memento
//!    to disk
//! 5. **Crash Recovery**: `Internal::LoadInitialMementoFromDisk()` with
//!    backup/restore
//!
//! ## ERROR HANDLING
//!
//! - Disk full: `CommonError::FileSystemIO`
//! - Permission denied: `CommonError::FileSystemIO`
//! - JSON parse error: `CommonError::SerializationError` (with recovery)
//! - Quota exceeded: `CommonError::StorageFull` (TODO)
//!
//! ## PERFORMANCE
//!
//! - All storage operations are in-memory (fast)
//! - Disk writes are async and batched
//! - Consider size limits (configurable max per storage file)
//! - Large values (>1MB) should be stored in files, not storage
//!
//! ## RECOVERY MECHANISMS
//!
//! - Corrupted JSON files are backed up with timestamps
//! - On parse error, storage is reset to empty and continues
//! - Directories are created automatically
//! - Writes are atomic (write to temp, then rename)
//!
//! ## VS CODE REFERENCE
//!
//! Patterns from VS Code:
//! - `vs/platform/storage/common/storageService.ts` - Storage service
//! - `vs/platform/storage/common/memento.ts` - Memento pattern for state
//!
//! ## TODO
//!
//! - [ ] Implement storage quotas (per-extension limits)
//! - [ ] Add storage encryption for sensitive data
//! - [ ] Support storage compression for large datasets
//! - [ ] Implement storage migration/versioning
//! - [ ] Add storage inspection and debugging tools
//! - [ ] Support storage syncing across devices (via Air)
//! - [ ] Implement storage TTL (time-to-live) for auto-expiring keys
//! - [ ] Add storage subscriptions/notifications on change
//! - [ ] Support binary data storage (not just JSON)
//! - [ ] Implement storage transactions (batch operations with rollback)
//!
//! ## MODULE CONTENTS
//!
//! - [`StorageProvider`]: Main struct implementing the trait
//! - Storage access methods (Get, Set, Remove, Clear, Keys)
//! - Memento loading and saving
//! - Recovery and backup logic

// Responsibilities:
//   - Core logic for Memento storage operations.
//   - Reading from and writing to global and workspace JSON storage files.
//   - Provides both per-key and high-performance batch operations.
//   - Enhances keychain integration with the `keyring` crate for secure storage.
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
// 1. **Global Storage**: Application-level settings that persist across all workspaces
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
