//! # StorageProvider (Environment)
//!
//! Implements the `StorageProvider` trait for `MountainEnvironment`. Contains
//! the core logic for Memento storage: reading from and writing to JSON
//! storage files on disk.
//!
//! ## Storage scopes
//!
//! - **Global** (`IsGlobalScope = true`) - application-level key-value store
//!   shared across all workspaces; persisted to `GlobalMementoPath`. Used for
//!   user preferences, extension state.
//! - **Workspace** (`IsGlobalScope = false`) - workspace-specific state;
//!   persisted to `WorkspaceMementoPath` (reloaded on workspace change via
//!   `UpdateWorkspaceMementoPathAndReload`). Used for workspace configs.
//!
//! ## Storage operations
//!
//! - `GetStorageValue(scope, key)` - reads from in-memory `HashMap`; returns
//!   `None` for missing or empty keys; rejects keys > 1 024 chars.
//! - `UpdateStorageValue(scope, key, value)` - inserts or removes key; rejects
//!   values > 10 MB; spawns async `SaveStorageToDisk` after each mutation.
//! - `GetAllStorage(scope)` - returns the full in-memory map as JSON.
//! - `SetAllStorage(scope, state)` - overwrites the full map and persists.
//!
//! ## Async persistence
//!
//! All disk writes go through `SaveStorageToDisk`, which is spawned via
//! `tokio::spawn` so the trait call returns immediately. The function creates
//! parent directories as needed and logs errors without propagating them
//! (fire-and-forget pattern). Writes are NOT yet atomic (temp+rename); that
//! is a known TODO.
//!
//! ## VS Code reference
//!
//! - `vs/platform/storage/common/storageService.ts`
//! - `vs/platform/storage/common/memento.ts`

use std::{
	collections::HashMap,
	path::PathBuf,
	sync::{
		Arc,
		Mutex,
		OnceLock,
		atomic::{AtomicBool, Ordering},
	},
};

use CommonLibrary::{Error::CommonError::CommonError, Storage::StorageProvider::StorageProvider};
use async_trait::async_trait;
use serde_json::Value;
use tokio::fs;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::dev_log;

/// Write-coalescing debouncer for storage scope.
/// Accumulates the latest data snapshot and schedules a single
/// disk write 100 ms after the first queued mutation in a burst.
struct StorageWriteDebouncer {
	Pending:Mutex<Option<(PathBuf, HashMap<String, Value>)>>,

	FlushScheduled:AtomicBool,
}

impl StorageWriteDebouncer {
	fn new() -> Arc<Self> { Arc::new(Self { Pending:Mutex::new(None), FlushScheduled:AtomicBool::new(false) }) }

	fn Queue(&self, Path:PathBuf, Data:HashMap<String, Value>, Debouncer:Arc<Self>) {
		if let Ok(mut Guard) = self.Pending.lock() {
			*Guard = Some((Path, Data));
		}

		if !self.FlushScheduled.swap(true, Ordering::AcqRel) {
			tokio::spawn(async move {
				tokio::time::sleep(std::time::Duration::from_millis(100)).await;

				let Item = {
					let mut Guard = Debouncer.Pending.lock().unwrap();

					Debouncer.FlushScheduled.store(false, Ordering::Release);

					Guard.take()
				};

				if let Some((StoragePath, StorageData)) = Item {
					SaveStorageToDisk(StoragePath, StorageData).await;
				}
			});
		}
	}
}

static GLOBAL_DEBOUNCER:OnceLock<Arc<StorageWriteDebouncer>> = OnceLock::new();

static WORKSPACE_DEBOUNCER:OnceLock<Arc<StorageWriteDebouncer>> = OnceLock::new();

fn GetGlobalDebouncer() -> Arc<StorageWriteDebouncer> {
	GLOBAL_DEBOUNCER.get_or_init(StorageWriteDebouncer::new).clone()
}

fn GetWorkspaceDebouncer() -> Arc<StorageWriteDebouncer> {
	WORKSPACE_DEBOUNCER.get_or_init(StorageWriteDebouncer::new).clone()
}

// TODO: storage quotas per extension, encryption for sensitive values,
// compression for large datasets, migration/versioning, atomic writes
// (temp+rename), storage change notifications/watchers, TTL / auto-expiry,
// binary data support, transaction (batch + rollback), sync via Air.
#[async_trait]
impl StorageProvider for MountainEnvironment {
	/// Retrieves a value from either global or workspace storage.
	/// Includes defensive validation to prevent invalid keys and invalid JSON.
	async fn GetStorageValue(&self, IsGlobalScope:bool, Key:&str) -> Result<Option<Value>, CommonError> {
		let ScopeName = if IsGlobalScope { "Global" } else { "Workspace" };

		dev_log!(
			"storage",
			"[StorageProvider] Getting value from {} scope for key: {}",
			ScopeName,
			Key
		);

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
			&self.ApplicationState.Configuration.MementoGlobalStorage
		} else {
			&self.ApplicationState.Configuration.MementoWorkspaceStorage
		};

		let StorageMapGuard = StorageMapMutex
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

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

		// Per-key updates fire at every workbench state change (sidebar
		// view state, panel layout, editor tab order, telemetry opt-ins).
		// Short-form + long-form both emit under `storage-verbose` so the
		// default log stays clean; `Trace=storage-verbose` restores
		// the original verbose tracing.
		if crate::IPC::DevLog::IsShort::Fn() {
			crate::dev_log!("storage-verbose", "update {} {}", ScopeName, Key);
		} else {
			dev_log!(
				"storage-verbose",
				"[StorageProvider] Updating value in {} scope for key: {}",
				ScopeName,
				Key
			);
		}

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
				self.ApplicationState.Configuration.MementoGlobalStorage.clone(),
				Some(
					self.ApplicationState
						.GlobalMementoPath
						.lock()
						.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
						.clone(),
				),
			)
		} else {
			(
				self.ApplicationState.Configuration.MementoWorkspaceStorage.clone(),
				self.ApplicationState
					.WorkspaceMementoPath
					.lock()
					.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
					.clone(),
			)
		};

		// Perform the in-memory update.
		let DataToSave = {
			let mut StorageMapGuard = StorageMapMutex
				.lock()
				.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

			if let Some(Value) = ValueToSet {
				StorageMapGuard.insert(Key, Value);
			} else {
				StorageMapGuard.remove(&Key);
			}

			StorageMapGuard.clone()
		};

		if let Some(StoragePath) = StoragePathOption {
			// Coalesce rapid writes: queue the latest snapshot and let the
			// debouncer emit a single disk write 100 ms after the burst ends.
			let Debouncer = if IsGlobalScope { GetGlobalDebouncer() } else { GetWorkspaceDebouncer() };

			Debouncer.Queue(StoragePath, DataToSave, Debouncer.clone());
		}

		Ok(())
	}

	/// Retrieves the entire storage map for a given scope.
	async fn GetAllStorage(&self, IsGlobalScope:bool) -> Result<Value, CommonError> {
		let ScopeName = if IsGlobalScope { "Global" } else { "Workspace" };

		dev_log!(
			"storage-verbose",
			"[StorageProvider] Getting all values from {} scope.",
			ScopeName
		);

		let StorageMapMutex = if IsGlobalScope {
			&self.ApplicationState.Configuration.MementoGlobalStorage
		} else {
			&self.ApplicationState.Configuration.MementoWorkspaceStorage
		};

		let StorageMapGuard = StorageMapMutex
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

		Ok(serde_json::to_value(&*StorageMapGuard)?)
	}

	/// Overwrites the entire storage map for a given scope and persists it.
	async fn SetAllStorage(&self, IsGlobalScope:bool, FullState:Value) -> Result<(), CommonError> {
		let ScopeName = if IsGlobalScope { "Global" } else { "Workspace" };

		dev_log!(
			"storage-verbose",
			"[StorageProvider] Setting all values for {} scope.",
			ScopeName
		);

		let DeserializedState:HashMap<String, Value> = serde_json::from_value(FullState)?;

		let (StorageMapMutex, StoragePathOption) = if IsGlobalScope {
			(
				self.ApplicationState.Configuration.MementoGlobalStorage.clone(),
				Some(
					self.ApplicationState
						.GlobalMementoPath
						.lock()
						.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
						.clone(),
				),
			)
		} else {
			(
				self.ApplicationState.Configuration.MementoWorkspaceStorage.clone(),
				self.ApplicationState
					.WorkspaceMementoPath
					.lock()
					.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
					.clone(),
			)
		};

		// Update in-memory state
		*StorageMapMutex
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)? = DeserializedState.clone();

		// Persist to disk via debouncer (coalesces rapid calls).
		if let Some(StoragePath) = StoragePathOption {
			let Debouncer = GetGlobalDebouncer();

			Debouncer.Queue(StoragePath, DeserializedState, Debouncer.clone());
		}

		Ok(())
	}
}

// --- Internal Helper Functions ---

/// An internal helper function to asynchronously write the storage map to a
/// file.
async fn SaveStorageToDisk(Path:PathBuf, Data:HashMap<String, Value>) {
	// Fires on every `storage:updateItems` that mutates the global map
	// (~50 per session during workbench boot alone). The failure path
	// below logs unconditionally; the success path is per-call noise.
	dev_log!(
		"storage-verbose",
		"[StorageProvider] Persisting storage to disk: {}",
		Path.display()
	);

	match serde_json::to_string_pretty(&Data) {
		Ok(JSONString) => {
			if let Some(ParentDirectory) = Path.parent() {
				if let Err(Error) = fs::create_dir_all(ParentDirectory).await {
					dev_log!(
						"storage",
						"error: [StorageProvider] Failed to create parent directory for '{}': {}",
						Path.display(),
						Error
					);

					return;
				}
			}

			if let Err(Error) = fs::write(&Path, JSONString).await {
				dev_log!(
					"storage",
					"error: [StorageProvider] Failed to write storage file to '{}': {}",
					Path.display(),
					Error
				);
			}
		},

		Err(Error) => {
			dev_log!(
				"storage",
				"error: [StorageProvider] Failed to serialize storage data for '{}': {}",
				Path.display(),
				Error
			);
		},
	}
}
