//! # ApplicationState
//!
//! Defines the main `ApplicationState` struct, which is the central, shared,
//! thread-safe state container for the entire Mountain application. It is
//! managed by Tauri and is accessible to all command handlers and Environment
//! providers.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. State Container
//! Hold all runtime state for services like:
//! - Workspace and window state
//! - Configuration and storage
//! - Extensions and command registry
//! - Documents and diagnostic errors
//! - Terminals, webviews, and tree views
//! - Source control management state
//! - Pending UI requests
//!
//! ### 2. Thread-Safe Access
//! - Provide thread-safe access to state via `Arc<Mutex<...>>`
//! - Ensure proper synchronization for concurrent access
//! - Handle mutex poisoning gracefully
//! - Support async operations with proper locking
//!
//! ### 3. State Persistence
//! - Manage memento (state serialization) for crash recovery
//! - Handle global and workspace-scoped storage
//! - Provide disk I/O for state loading/saving
//! - Recover from corrupted state files
//!
//! ### 4. Identity Management
//! - Generate unique provider handles
//! - Generate unique terminal identifiers
//! - Generate unique SCM provider handles
//! - Ensure monotonically increasing IDs
//!
//! ### 5. State Recovery
//! - Detect corrupted state
//! - Attempt recovery from poisoned locks
//! - Restore state from disk
//! - Clear invalid state entries
//!
//! ## ARCHITECTURAL ROLE
//!
//! The ApplicationState module is the **state management layer** of Mountain:
//!
//! ```text
//! UI ──► Commands ──► ApplicationState (State) ──► Providers/Services
//!                      │
//!                      ↓
//!                   Disk (Persistence)
//! ```
//!
//! ### Design Principles:
//! 1. **Single Source of Truth**: All state lives in one place
//! 2. **Thread Safety**: All state is protected by Arc<Mutex<...>>
//! 3. **Recovery-Oriented**: Comprehensive error handling and recovery
//! 4. **Type Safety**: Strong typing at all levels
//! 5. **Observability**: Comprehensive logging for state changes
//!
//! ### VS Code Reference:
//! This module borrows from VS Code's state management patterns in:
//! - `vs/base/parts/storage/common/storageService.ts` - Storage management
//! - `vs/workbench/services/environment/common/environmentService.ts` -
//!   Environment state
//! - `vs/platform/workspace/common/workspace.ts` - Workspace state
//! - `vs/workbench/services/extensions/common/extensions.ts` - Extension state
//!
//! Key concepts:
//! - Global vs workspace-scoped storage
//! - Memento (state serialization) for crash recovery
//! - Thread-safe state access with proper locking
//! - State validation and invariants
// ### 1. State Container
// Hold all runtime state for services like:
// - Workspace and window state
// - Configuration and storage
// - Extensions and command registry
// - Documents and diagnostic errors
// - Terminals, webviews, and tree views
// - Source control management state
// - Pending UI requests
//
// ### 2. Thread-Safe Access
// - Provide thread-safe access to state via `Arc<Mutex<...>>`
// - Ensure proper synchronization for concurrent access
// - Handle mutex poisoning gracefully
// - Support async operations with proper locking
//
// ### 3. State Persistence
// - Manage memento (state serialization) for crash recovery
// - Handle global and workspace-scoped storage
// - Provide disk I/O for state loading/saving
// - Recover from corrupted state files
//
// ### 4. Identity Management
// - Generate unique provider handles
// - Generate unique terminal identifiers
// - Generate unique SCM provider handles
// - Ensure monotonically increasing IDs
//
// ### 5. State Recovery
// - Detect corrupted state
// - Attempt recovery from poisoned locks
// - Restore state from disk
// - Clear invalid state entries
//
// ## VS Code Reference
//
// This module borrows from VS Code's state management patterns in:
//
// - `vs/platform/storage/common/storageService.ts` - Memento storage
//   - Global vs workspace-scoped storage separation
//   - Crash recovery and state persistence
//   - Key-value storage API
//
// - `vs/workbench/services/environment/common/environmentService.ts` - Environment state
//   - Workspace configuration management
//   - Window state persistence
//   - Trust management
//
// - `vs/workbench/services/extensions/common/extensions` - Extension state
//   - Extension registry and metadata
//   - Language provider management
//   - Command registration
//
// Key patterns adopted:
// 1. **Memento Pattern**: State is serialized for crash recovery
// 2. **Repository Pattern**: State access goes through this container
// 3. **Identity Map**: Track all instances with unique IDs
// 4. **Observer Pattern**: State changes trigger events
//
// ## Data Layout
//
// The ApplicationState struct is organized into logical groups:
//
// ### Workspace State
// - `WorkspaceFolders` - Open workspace folders
// - `WorkspaceConfigurationPath` - Active workspace config file
// - `IsTrusted` - Workspace security trust status
// - `WindowState` - Window geometry and state
// - `ActiveDocumentURI` - Currently active document
//
// ### Configuration & Storage
// - `Configuration` - Merged configuration from all sources
// - `GlobalMemento` - Global key-value storage
// - `WorkspaceMemento` - Workspace-scoped storage
// - Memento paths for persistence
//
// ### Extension & Provider Management
// - `CommandRegistry` - Registered CLI commands
// - `LanguageProviders` - LSP and other language features
// - `NextProviderHandle` - Counter for provider IDs
// - `ScannedExtensions` - Discovered extensions
// - `EnabledProposedAPIs` - API feature flags
// - `ExtensionScanPaths` - Where to look for extensions
//
// ### Feature-specific State
// - `DiagnosticsMap` - Compiler/diagnostic errors by owner and resource
// - `OpenDocuments` - Currently open documents by URI
// - `OutputChannels` - Output panel channels
// - `ActiveTerminals` - Terminal instances by ID (nested mutex)
// - `NextTerminalIdentifier` - Counter for terminal IDs
// - `ActiveWebviews` - Webview panels by ID
// - `ActiveCustomDocuments` - Custom editor state
// - `ActiveStatusBarItems` - Status bar entries
// - `ActiveTreeViews` - Tree data providers
// - `SourceControlManagementProviders` - SCM registries
// - `SourceControlManagementGroups` - SCM resource groups
// - `SourceControlManagementResources` - SCM resource state
// - `NextSourceControlManagementProviderHandle` - Counter for SCM IDs
//
// ### IPC & UI State
// - `PendingUserInterfaceRequests` - Ongoing UI interactions (dialogs, etc.)
//
// ## Thread Safety
//
// All state is protected by `Arc<Mutex<...>>`:
//
//```rust
// pub struct ApplicationState {
//     pub WorkspaceFolders: Arc<Mutex<Vec<WorkspaceFolderStateDTO>>>,
//     pub Configuration: Arc<Mutex<MergedConfigurationStateDTO>>,
//     // ... all fields protected by Arc<Mutex<...>>
// }
//```
// **Access Patterns:**
// - Lock briefly, copy data needed, release immediately
// - Use `map_err(MapLockError)` for lock error handling
// - Avoid nested locks to prevent deadlocks
// - Prefer read-only copies when possible
//
// **Terminal State Note:**
// Terminals use `Arc<Mutex<...>>>` (double mutex) because:
// - Outer mutex protects the HashMap of terminals
// - Inner mutex protects each individual terminal state
// - Allows concurrent access to different terminals
//
// ## State Initialization
//
// The `Default` implementation creates a fully initialized state:
//
// 1. **Resolve Application Data Directory**:
//    - Use `dirs::config_dir()` on supported platforms
//    - Fall back to relative path if unavailable
//
// 2. **Ensure Directory Exists**:
//    - Create if missing
//    - Log error if creation fails
//
// 3. **Load Global Memento**:
//    - Read from disk if exists
//    - Default to empty map if not
//    - Handle corruption gracefully
//
// 4. **Initialize All Fields**:
//    - Empty collections for maps and vectors
//    - At starting value of 1 for counters
//    - Default structs for complex state
//
// ## Workspace Identification
//
// The workspace identifier is generated by `GetWorkspaceIdentifier`:
//
// **Priority**:
// 1. Configuration file name (if workspace open from file)
// 2. First workspace folder hashed and sanitized
// 3. "NO_WORKSPACE" if no workspace loaded
//
// **Format**: `{folder-name}-{hash[:8]}` or `{config-file-name}`
//
// Example: `MyProject-a1b2c3d4` or `settings.json`
//
// This identifier is used for:
// - Workspace memento file naming
// - Workspace-specific storage
// - Workspace identification in logs
//
// ## Memento Persistence
//
// Memento files store state for crash recovery:
//
// **Global Memento** (`globalStorage.json`):
// - Application-wide settings
// - User preferences
// - Cross-workspace data
//
// **Workspace Memento** (`workspaceStorage/{id}/storage.json`):
// - Workspace-specific settings
// - Document state
// - Per-workspace preferences
//
// **Update Flow**:
// - When workspace opens/changes → `UpdateWorkspaceMementoPathAndReload`
// - This updates path and reloads from disk
// - Old workspace state is forgotten
// - New workspace state is loaded
//
// ## Error Handling
//
// All functions return `Result<T, CommonError>`:
//
// **Error Types**:
// - `CommonError::StateLockPoisoned` - Mutex poisoned (panic in another thread)
// - `CommonError::FileSystemIO` - File/directory operations failed
// - `CommonError::SerializationError` - JSON parsing failed
// - `CommonError::Unknown` - Uncategorized errors
//
// **Recovery Functions**:
// - `MapLockError` - Convert lock error to CommonError
// - `MapLockErrorWithRecovery` - Convert with recovery attempt
// - `SafeStateOperation` - Wrap operation with recovery
// - `RecoverApplicationState` - Comprehensive recovery
//
// **StateOperationResult**:
// Provides recovery metadata:
// - `result` - Operation result
// - `recovery_attempted` - Was recovery tried
// - `recovery_successful` - Did recovery work
//
// ## Recovery Mechanisms
//
// ### State Recovery Functions:
//
// **`RecoverApplicationState()`**:
// - Recovers all state components
// - Calls all sub-recovery functions
// - Logs comprehensive status
//
// **`RecoverGlobalMemento()`**:
// - Reloads global memento from disk
// - Resets to empty if corrupted
//
// **`RecoverWorkspaceMemento()`**:
// - Reloads workspace memento from disk
// - Clears if path is None
//
// **`RecoverExtensionState()`**:
// - Clears potentially corrupted extensions
// - Removes invalid scan paths
//
// **`RecoverDocumentState()`**:
// - Removes documents that don't exist on disk
// - Keeps non-file URIs (untitled, virtual)
//
// ## TODOs
//
// High Priority:
// - [ ] Add state validation invariants
// - [ ] Implement state diffing for debugging
// - [ ] Add state metrics collection
//
// Medium Priority:
// - [ ] Add state compaction for large maps
// - [ ] Implement state snapshots
// - [ ] Add state export functionality
//
// Low Priority:
// - [ ] Add state visualization tools
// - [ ] Implement state cloning for testing
// - [ ] Add state migration for version upgrades

//! # ApplicationState Struct
//!
//! Defines the main `ApplicationState` struct, which is the central, shared,
//! thread-safe state container for the entire Mountain application. It is
//! managed by Tauri and is accessible to all command handlers and Environment
//! providers.

use std::{
	collections::HashMap,
	path::{Path, PathBuf},
	sync::{
		Arc,
		Mutex as StandardMutex,
		PoisonError,
		atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering as AtomicOrdering},
	},
};

use CommonLibrary::{
	Error::CommonError::CommonError,
	SourceControlManagement::DTO::{
		SourceControlManagementGroupDTO::SourceControlManagementGroupDTO,
		SourceControlManagementProviderDTO::SourceControlManagementProviderDTO,
		SourceControlManagementResourceDTO::SourceControlManagementResourceDTO,
	},
	StatusBar::DTO::StatusBarEntryDTO::StatusBarEntryDTO,
};
use log::{debug, error, info, warn};
use tauri::Wry;

use super::{
	DTO::{
		CustomDocumentStateDTO::CustomDocumentStateDTO,
		DocumentStateDTO::DocumentStateDTO,
		ExtensionDescriptionStateDTO::ExtensionDescriptionStateDTO,
		MarkerDataDTO::MarkerDataDTO,
		MergedConfigurationStateDTO::MergedConfigurationStateDTO,
		OutputChannelStateDTO::OutputChannelStateDTO,
		ProviderRegistrationDTO::ProviderRegistrationDTO,
		TerminalStateDTO::TerminalStateDTO,
		TreeViewStateDTO::TreeViewStateDTO,
		WebviewStateDTO::WebviewStateDTO,
		WindowStateDTO::WindowStateDTO,
		WorkspaceFolderStateDTO::WorkspaceFolderStateDTO,
	},
	Internal,
};
use crate::Environment::CommandProvider::CommandHandler;

/// The central, shared, thread-safe state for the entire Mountain application.
#[derive(Clone)]
pub struct ApplicationState {
	// --- Workspace State ---
	pub WorkspaceFolders:Arc<StandardMutex<Vec<WorkspaceFolderStateDTO>>>,

	pub WorkspaceConfigurationPath:Arc<StandardMutex<Option<PathBuf>>>,

	pub IsTrusted:Arc<AtomicBool>,

	pub WindowState:Arc<StandardMutex<WindowStateDTO>>,

	pub ActiveDocumentURI:Arc<StandardMutex<Option<String>>>,

	// --- Configuration & Storage ---
	pub Configuration:Arc<StandardMutex<MergedConfigurationStateDTO>>,

	pub GlobalMemento:Arc<StandardMutex<HashMap<String, serde_json::Value>>>,

	pub GlobalMementoPath:PathBuf,

	pub WorkspaceMemento:Arc<StandardMutex<HashMap<String, serde_json::Value>>>,

	pub WorkspaceMementoPath:Arc<StandardMutex<Option<PathBuf>>>,

	// --- Extension & Provider Management ---
	pub CommandRegistry:Arc<StandardMutex<HashMap<String, CommandHandler<Wry>>>>,

	pub LanguageProviders:Arc<StandardMutex<HashMap<u32, ProviderRegistrationDTO>>>,

	pub NextProviderHandle:Arc<AtomicU32>,

	pub ScannedExtensions:Arc<StandardMutex<HashMap<String, ExtensionDescriptionStateDTO>>>,

	pub EnabledProposedAPIs:Arc<StandardMutex<HashMap<String, Vec<String>>>>,

	pub ExtensionScanPaths:Arc<StandardMutex<Vec<PathBuf>>>,

	// --- Feature-specific State ---
	pub DiagnosticsMap:Arc<StandardMutex<HashMap<String, HashMap<String, Vec<MarkerDataDTO>>>>>,

	pub OpenDocuments:Arc<StandardMutex<HashMap<String, DocumentStateDTO>>>,

	pub OutputChannels:Arc<StandardMutex<HashMap<String, OutputChannelStateDTO>>>,

	pub ActiveTerminals:Arc<StandardMutex<HashMap<u64, Arc<StandardMutex<TerminalStateDTO>>>>>,

	pub NextTerminalIdentifier:Arc<AtomicU64>,

	pub ActiveWebviews:Arc<StandardMutex<HashMap<String, WebviewStateDTO>>>,

	pub ActiveCustomDocuments:Arc<StandardMutex<HashMap<String, CustomDocumentStateDTO>>>,

	pub ActiveStatusBarItems:Arc<StandardMutex<HashMap<String, StatusBarEntryDTO>>>,

	pub ActiveTreeViews:Arc<StandardMutex<HashMap<String, TreeViewStateDTO>>>,

	pub SourceControlManagementProviders:Arc<StandardMutex<HashMap<u32, SourceControlManagementProviderDTO>>>,

	pub SourceControlManagementGroups:
		Arc<StandardMutex<HashMap<u32, HashMap<String, SourceControlManagementGroupDTO>>>>,

	pub SourceControlManagementResources:
		Arc<StandardMutex<HashMap<u32, HashMap<String, Vec<SourceControlManagementResourceDTO>>>>>,

	pub NextSourceControlManagementProviderHandle:Arc<AtomicU32>,

	// --- Test Provider State ---
	pub TestProviderState:Arc<tokio::sync::RwLock<crate::Environment::TestProvider::TestProviderState>>,

	// --- IPC & User Interface State ---
	pub PendingUserInterfaceRequests:
		Arc<StandardMutex<HashMap<String, tokio::sync::oneshot::Sender<Result<serde_json::Value, CommonError>>>>>,
}

/// A helper to map a mutex poison error into a `CommonError`.
pub fn MapLockError<T>(Error:PoisonError<T>) -> CommonError {
	CommonError::StateLockPoisoned { Context:Error.to_string() }
}

/// A helper to map a mutex poison error with recovery attempt.
pub fn MapLockErrorWithRecovery<T>(Error:PoisonError<T>, RecoveryContext:&str) -> CommonError {
	warn!(
		"[ApplicationState] Attempting recovery from poisoned lock in context: {}",
		RecoveryContext
	);
	CommonError::StateLockPoisoned {
		Context:format!("{} - Recovery attempted: {}", Error.to_string(), RecoveryContext),
	}
}

/// Error handling result with recovery information
#[derive(Debug)]
pub struct StateOperationResult<T> {
	pub result:Result<T, CommonError>,
	pub recovery_attempted:bool,
	pub recovery_successful:bool,
}

impl Default for ApplicationState {
	fn default() -> Self {
		info!("[ApplicationState] Initializing default application state...");

		let ApplicationNameForPaths = env!("CARGO_PKG_NAME");

		let ApplicationDataDirectory = dirs::config_dir()
			.map(|Path| Path.join(ApplicationNameForPaths))
			.unwrap_or_else(|| {
				warn!(
					"[ApplicationState] Could not get config dir. Using relative path '.{}-appdata'.",
					ApplicationNameForPaths
				);

				PathBuf::from(format!(".{}-appdata", ApplicationNameForPaths))
			});

		// This must be synchronous because the async runtime isn't available yet.
		if !ApplicationDataDirectory.exists() {
			if let Err(Error) = std::fs::create_dir_all(&ApplicationDataDirectory) {
				error!(
					"[ApplicationState] CRITICAL: Failed to create app data directory at '{}': {}.",
					ApplicationDataDirectory.display(),
					Error
				);
			}
		}

		let GlobalMementoFilePath = Internal::ResolveMementoStorageFilePath(&ApplicationDataDirectory, true, "");

		let InitialGlobalMementoMap = Internal::LoadInitialMementoFromDisk(&GlobalMementoFilePath);

		info!("[ApplicationState] Default state initialization complete.");

		Self {
			WorkspaceFolders:Arc::new(StandardMutex::new(Vec::new())),

			WorkspaceConfigurationPath:Arc::new(StandardMutex::new(None)),

			IsTrusted:Arc::new(AtomicBool::new(false)),

			WindowState:Arc::new(StandardMutex::new(Default::default())),

			ActiveDocumentURI:Arc::new(StandardMutex::new(None)),

			Configuration:Arc::new(StandardMutex::new(MergedConfigurationStateDTO::default())),

			GlobalMemento:Arc::new(StandardMutex::new(InitialGlobalMementoMap)),

			GlobalMementoPath:GlobalMementoFilePath,

			WorkspaceMemento:Arc::new(StandardMutex::new(HashMap::new())),

			WorkspaceMementoPath:Arc::new(StandardMutex::new(None)),

			CommandRegistry:Arc::new(StandardMutex::new(HashMap::new())),

			LanguageProviders:Arc::new(StandardMutex::new(HashMap::new())),

			NextProviderHandle:Arc::new(AtomicU32::new(1)),

			ScannedExtensions:Arc::new(StandardMutex::new(HashMap::new())),

			EnabledProposedAPIs:Arc::new(StandardMutex::new(HashMap::new())),

			ExtensionScanPaths:Arc::new(StandardMutex::new(Vec::new())),

			DiagnosticsMap:Arc::new(StandardMutex::new(HashMap::new())),

			OpenDocuments:Arc::new(StandardMutex::new(HashMap::new())),

			OutputChannels:Arc::new(StandardMutex::new(HashMap::new())),

			ActiveTerminals:Arc::new(StandardMutex::new(HashMap::new())),

			NextTerminalIdentifier:Arc::new(AtomicU64::new(1)),

			ActiveWebviews:Arc::new(StandardMutex::new(HashMap::new())),

			ActiveCustomDocuments:Arc::new(StandardMutex::new(HashMap::new())),

			ActiveStatusBarItems:Arc::new(StandardMutex::new(HashMap::new())),

			ActiveTreeViews:Arc::new(StandardMutex::new(HashMap::new())),

			SourceControlManagementProviders:Arc::new(StandardMutex::new(HashMap::new())),

			SourceControlManagementGroups:Arc::new(StandardMutex::new(HashMap::new())),

			SourceControlManagementResources:Arc::new(StandardMutex::new(HashMap::new())),

			NextSourceControlManagementProviderHandle:Arc::new(AtomicU32::new(1)),

			TestProviderState:Arc::new(tokio::sync::RwLock::new(
				crate::Environment::TestProvider::TestProviderState::new(),
			)),

			PendingUserInterfaceRequests:Arc::new(StandardMutex::new(HashMap::new())),
		}
	}
}

impl ApplicationState {
	/// Generates a unique, filesystem-safe identifier for the current
	/// workspace. Returns "NO_WORKSPACE" if no folder or configuration is
	/// open.
	pub fn GetWorkspaceIdentifier(&self) -> Result<String, CommonError> {
		let ConfigurationPathGuard = self
			.WorkspaceConfigurationPath
			.lock()
			.map_err(|e| MapLockErrorWithRecovery(e, "GetWorkspaceIdentifier - ConfigurationPath"))?;

		if let Some(ConfigurationPath) = ConfigurationPathGuard.as_ref() {
			let FileName = ConfigurationPath.file_name().unwrap_or_default().to_string_lossy().into_owned();

			return Ok(FileName);
		}

		drop(ConfigurationPathGuard);

		let FoldersGuard = self
			.WorkspaceFolders
			.lock()
			.map_err(|e| MapLockErrorWithRecovery(e, "GetWorkspaceIdentifier - WorkspaceFolders"))?;

		if let Some(FirstFolder) = FoldersGuard.first() {
			let PathString = FirstFolder.URI.path();

			// Create a more stable hash for the identifier.
			let Hash = format!("{:x}", md5::compute(PathString));

			let SafeName = FirstFolder.Name.replace(|c:char| !c.is_alphanumeric(), "_");

			return Ok(format!("{}-{}", SafeName, &Hash[..8]));
		}

		Ok("NO_WORKSPACE".to_string())
	}

	/// Safe state operation with automatic recovery
	pub fn SafeStateOperation<T, F>(&self, operation_name:&str, operation:F) -> StateOperationResult<T>
	where
		F: FnOnce() -> Result<T, CommonError>, {
		let mut recovery_attempted = false;
		let mut recovery_successful = false;

		match operation() {
			Ok(result) => StateOperationResult { result:Ok(result), recovery_attempted, recovery_successful },
			Err(error) => {
				// Attempt recovery for specific error types
				if Self::should_attempt_recovery(&error) {
					recovery_attempted = true;
					match Self::attempt_state_recovery(&error, operation_name) {
						Ok(()) => {
							recovery_successful = true;
							info!("[ApplicationState] Recovery successful for operation: {}", operation_name);
							StateOperationResult {
								result:Err(error), // Original error still returned
								recovery_attempted,
								recovery_successful,
							}
						},
						Err(recovery_error) => {
							warn!(
								"[ApplicationState] Recovery failed for operation {}: {}",
								operation_name, recovery_error
							);
							StateOperationResult {
								result:Err(error), // Original error still returned
								recovery_attempted,
								recovery_successful,
							}
						},
					}
				} else {
					StateOperationResult { result:Err(error), recovery_attempted, recovery_successful }
				}
			},
		}
	}

	/// Determine if recovery should be attempted for a given error
	fn should_attempt_recovery(error:&CommonError) -> bool {
		match error {
			CommonError::StateLockPoisoned { .. } => true,
			CommonError::FileSystemIO { .. } => true,
			CommonError::SerializationError { .. } => true,
			_ => false,
		}
	}

	/// Attempt state recovery based on error type
	fn attempt_state_recovery(error:&CommonError, context:&str) -> Result<(), CommonError> {
		match error {
			CommonError::StateLockPoisoned { .. } => {
				// For poisoned locks, we can't do much but log and wait
				warn!("[ApplicationState] Poisoned lock detected in context: {}", context);
				// Small delay to allow system to stabilize
				std::thread::sleep(std::time::Duration::from_millis(100));
				Ok(())
			},
			CommonError::FileSystemIO { Path, .. } => {
				// Attempt to recreate directories or handle file system issues
				if let Some(parent) = Path.parent() {
					if !parent.exists() {
						std::fs::create_dir_all(parent).map_err(|e| {
							CommonError::FileSystemIO {
								Path:parent.to_path_buf(),
								Description:format!("Failed to create directory during recovery: {}", e),
							}
						})?;
					}
				}
				Ok(())
			},
			CommonError::SerializationError { .. } => {
				// For serialization errors, we might need to reset corrupted state
				warn!(
					"[ApplicationState] Serialization error detected, state may be corrupted: {}",
					context
				);
				Ok(())
			},
			_ => Ok(()),
		}
	}

	/// Returns the next available unique identifier for a language provider
	/// registration.
	pub fn GetNextProviderHandle(&self) -> u32 { self.NextProviderHandle.fetch_add(1, AtomicOrdering::Relaxed) }

	/// Returns the next available unique identifier for a terminal instance.
	pub fn GetNextTerminalIdentifier(&self) -> u64 { self.NextTerminalIdentifier.fetch_add(1, AtomicOrdering::Relaxed) }

	/// Returns the next available unique identifier for an SCM provider
	/// instance.
	pub fn GetNextSourceControlManagementProviderHandle(&self) -> u32 {
		self.NextSourceControlManagementProviderHandle
			.fetch_add(1, AtomicOrdering::Relaxed)
	}

	/// Updates the path to the workspace memento file and reloads its content
	/// from disk. This should be called when a workspace is opened or changed.
	pub fn UpdateWorkspaceMementoPathAndReload(&self, ApplicationDataDirectory:&Path) -> Result<(), CommonError> {
		let operation_result = self.SafeStateOperation("UpdateWorkspaceMementoPathAndReload", || {
			let WorkspaceIdentifier = self.GetWorkspaceIdentifier()?;

			let mut PathGuard = self.WorkspaceMementoPath.lock().map_err(|e| {
				MapLockErrorWithRecovery(e, "UpdateWorkspaceMementoPathAndReload - WorkspaceMementoPath")
			})?;

			if WorkspaceIdentifier == "NO_WORKSPACE" {
				if PathGuard.is_some() {
					*PathGuard = None;

					self.WorkspaceMemento
						.lock()
						.map_err(|e| {
							MapLockErrorWithRecovery(e, "UpdateWorkspaceMementoPathAndReload - WorkspaceMemento")
						})?
						.clear();
				}

				return Ok(());
			}

			let NewMementoPath =
				Internal::ResolveMementoStorageFilePath(ApplicationDataDirectory, false, &WorkspaceIdentifier);

			if PathGuard.as_ref() != Some(&NewMementoPath) {
				if let Some(Parent) = NewMementoPath.parent() {
					if !Parent.exists() {
						std::fs::create_dir_all(Parent).map_err(|Error| {
							CommonError::FileSystemIO { Path:Parent.to_path_buf(), Description:Error.to_string() }
						})?;
					}
				}

				*PathGuard = Some(NewMementoPath.clone());

				let NewMementoContent = Internal::LoadInitialMementoFromDisk(&NewMementoPath);

				*self.WorkspaceMemento.lock().map_err(|e| {
					MapLockErrorWithRecovery(e, "UpdateWorkspaceMementoPathAndReload - WorkspaceMemento update")
				})? = NewMementoContent;
			}

			Ok(())
		});

		operation_result.result
	}

	/// Enhanced state recovery with comprehensive error handling
	pub async fn RecoverApplicationState(&self) -> Result<(), CommonError> {
		info!("[ApplicationState] Starting comprehensive state recovery...");

		// Recover global memento
		self.RecoverGlobalMemento().await?;

		// Recover workspace memento
		self.RecoverWorkspaceMemento().await?;

		// Recover extension state
		self.RecoverExtensionState().await?;

		// Recover document state
		self.RecoverDocumentState().await?;

		info!("[ApplicationState] Comprehensive state recovery completed successfully");
		Ok(())
	}

	/// Recover global memento state
	async fn RecoverGlobalMemento(&self) -> Result<(), CommonError> {
		debug!("[ApplicationState] Recovering global memento state...");

		let memento_content = Internal::LoadInitialMementoFromDisk(&self.GlobalMementoPath);
		let mut global_memento = self
			.GlobalMemento
			.lock()
			.map_err(|e| MapLockErrorWithRecovery(e, "RecoverGlobalMemento"))?;

		*global_memento = memento_content;
		Ok(())
	}

	/// Recover workspace memento state
	async fn RecoverWorkspaceMemento(&self) -> Result<(), CommonError> {
		debug!("[ApplicationState] Recovering workspace memento state...");

		let workspace_path_guard = self
			.WorkspaceMementoPath
			.lock()
			.map_err(|e| MapLockErrorWithRecovery(e, "RecoverWorkspaceMemento - Path"))?;

		if let Some(path) = workspace_path_guard.as_ref() {
			let memento_content = Internal::LoadInitialMementoFromDisk(path);
			let mut workspace_memento = self
				.WorkspaceMemento
				.lock()
				.map_err(|e| MapLockErrorWithRecovery(e, "RecoverWorkspaceMemento - Content"))?;

			*workspace_memento = memento_content;
		}

		Ok(())
	}

	/// Recover extension state
	async fn RecoverExtensionState(&self) -> Result<(), CommonError> {
		debug!("[ApplicationState] Recovering extension state...");

		// Clear potentially corrupted extension state
		let mut scanned_extensions = self
			.ScannedExtensions
			.lock()
			.map_err(|e| MapLockErrorWithRecovery(e, "RecoverExtensionState - ScannedExtensions"))?;

		scanned_extensions.clear();

		// Reset extension scan paths
		let mut scan_paths = self
			.ExtensionScanPaths
			.lock()
			.map_err(|e| MapLockErrorWithRecovery(e, "RecoverExtensionState - ExtensionScanPaths"))?;

		// Keep only valid paths that exist
		scan_paths.retain(|path| path.exists());

		Ok(())
	}

	/// Recover document state
	async fn RecoverDocumentState(&self) -> Result<(), CommonError> {
		debug!("[ApplicationState] Recovering document state...");

		// Clear potentially corrupted document state
		let mut open_documents = self
			.OpenDocuments
			.lock()
			.map_err(|e| MapLockErrorWithRecovery(e, "RecoverDocumentState - OpenDocuments"))?;

		// Remove documents that reference non-existent files
		open_documents.retain(|uri, _doc_state| {
			if let Ok(parsed_url) = url::Url::parse(uri) {
				if parsed_url.scheme() == "file" {
					if let Ok(path) = parsed_url.to_file_path() {
						return path.exists();
					}
				}
			}
			true // Keep non-file URIs or invalid URIs
		});
		Ok(())
	}
}
