//! # ApplicationState Module
//!
//! ## Responsibilities
//!
//! This module defines the central, shared, thread-safe state for the entire
//! Mountain application. It serves as the **single source of truth** for all
//! runtime state including:
//!
//! **State Management:**
//! - Main `ApplicationState` struct with all runtime state
//! - Thread-safe access via `Arc<Mutex<...>>` wrappers
//! - State transitions and invariants enforcement
//! - Recovery mechanisms for corrupted state
//! - Disk persistence for memento (workspace and global) state
//!
//! **Data Structures:**
//! - Data Transfer Objects (DTOs) for various components
//! - Type-safe state representations
//! - Serialization/deserialization support
//! - Validation for state updates
//!
//! **Internal Utilities:**
//! - File I/O and path resolution
//! - State initialization and loading
//! - Extension scanning and population
//! - URL serialization helpers
//! - Recovery utilities for corrupted state
//!
//! ## Architectural Role
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
//
//! ## Key Components
//!
//! ### ApplicationState
//! The central state struct containing all runtime state:
//!
//! **Workspace State:**
//! - `WorkspaceFolders` - Currently open workspace folders
//! - `WorkspaceConfigurationPath` - Path to workspace config
//! - `IsTrusted` - Workspace trust status
//! - `WindowState` - Window size, position, etc.
//! - `ActiveDocumentURI` - Currently active document
//!
//! **Configuration & Storage:**
//! - `Configuration` - Merged configuration state
//! - `GlobalMemento` - Global storage (keys → values)
//! - `WorkspaceMemento` - Workspace-scoped storage
//! - Memento paths for persistence
//!
//! **Extension & Provider Management:**
//! - `CommandRegistry` - Registered commands
//! - `LanguageProviders` - Language feature providers
//! - `ScannedExtensions` - Discovered extensions
//! - `ExtensionScanPaths` - Paths to search for extensions
//! - `EnabledProposedAPIs` - Proposed API permissions
//!
//! **Feature-specific State:**
//! - `DiagnosticsMap` - Compiler/diagnostic errors
//! - `OpenDocuments` - Currently open documents
//! - `OutputChannels` - Output channel state
//! - `ActiveTerminals` - Active terminal instances
//! - `ActiveWebviews` - Active webview panels
//! - `ActiveCustomDocuments` - Custom editor state
//! - `ActiveStatusBarItems` - Status bar entries
//! - `ActiveTreeViews` - Tree view providers
//! - `SourceControlManagement*` - SCM provider state
//!
//! **IPC & UI State:**
//! - `PendingUserInterfaceRequest` - Pending UI interactions (dialogs, etc.)
//!
//! ### DTOs
//! Type-safe representations of state components:
//! - `WorkspaceFolderStateDTO` - Workspace folder representation
//! - `DocumentStateDTO` - Document metadata and content
//! - `ExtensionDescriptionStateDTO` - Extension metadata
//! - `TreeViewStateDTO` - Tree view state
//! - `WebviewStateDTO` - Webview panel state
//! - etc.
//!
//! ### Internal
//! Helper functions for state management:
//! - `LoadInitialMementoFromDisk` - Load persistent state from disk
//! - `ResolveMementoStorageFilePath` - Resolve storage paths
//! - `ScanAndPopulateExtensions` - Scan for extensions
//! - `AnalyzeTextLinesAndEOL` - Text content analysis
//! - Recovery utilities and error handling
//!
//! ## Thread Safety
//!
//! All state is protected by `Arc<Mutex<...>>` for thread-safe access:
//!
//! ```rust
//! pub struct ApplicationState {
//! 	pub WorkspaceFolders:Arc<Mutex<Vec<WorkspaceFolderStateDTO>>>,
//! 	pub Configuration:Arc<Mutex<MergedConfigurationStateDTO>>,
//! 	// ... all fields follow this pattern
//! }
//! ```
//!
//! **Best Practices:**
//! - Always lock mutexes before accessing state
//! - Use `map_err(MapLockError)` for lock error handling
//! - Release locks as soon as possible (use RAII)
//! - Avoid nested locks to prevent deadlocks
//!
//! ## State Persistence
//!
//! ### Memento System
//! Mountain uses a memento system for state persistence:
//!
//! **Global Memento:**
//! - Lives at `{AppData}/globalStorage.json`
//! - Contains global application state
//! - Persists across workspace changes
//!
//! **Workspace Memento:**
//! - Lives at `{AppData}/workspaceStorage/{workspace-id}/storage.json`
//! - Contains workspace-specific state
//! - Changes when workspace changes
//!
//! **Recovery:**
//! - Corrupted JSON files are backed up with timestamps
//! - Empty maps are returned on parse errors
//! - Directories are created automatically
//!
//! ## Error Handling
//!
//! All state operations return `Result<T, CommonError>` where:
//! - `Ok(T)` - Successful operation with result
//! - `Err(CommonError)` - Error with context
//!
//! Error types:
//! - `StateLockPoisoned` - Mutex was poisoned (another thread panicked)
//! - `FileSystemIO` - File system operations failed
//! - `SerializationError` - JSON parsing failed
//! - `Unknown` - Uncategorized errors
//!
//! Recovery mechanisms:
//! - `MapLockErrorWithRecovery` - Attempt recovery from poisoned locks
//! - `SafeStateOperation` - Safe state operation with recovery
//! - `RecoverApplicationState` - Comprehensive state recovery
//!
//! ## State Transitions
//!
//! Valid state transitions:
//!
//! ```text
//! Uninitialized → Default State → Workspace Loaded → Workspace Unloaded
//!      ↓                ↓                 ↓                  ↓
//!    Load           Recover          Update Memento      Clean Up
//! ```
//!
//! **Invariants:**
//! - Workspace identifier is computed from folder or config
//! - Memento paths are synced with workspace changes
//! - Provider handles are monotonically increasing
//! - Unique IDs are never reused
//!
//! ## TODOs
//!
//! High Priority:
//! - [ ] Add state migration support for version changes
//! - [ ] Implement state diffing for better debugging
//! - [ ] Add state snapshots for rollback support
//!
//! Medium Priority:
//! - [ ] Add state compression for large workspaces
//! - [ ] Implement incremental memento updates
//! - [ ] Add state validation on load
//!
//! Low Priority:
//! - [ ] Add state analytics and metrics
//! - [ ] Implement state cloning for testing
//! - [ ] Add state export/import functionality

// --- Public Modules ---

/// State management sub-modules.
pub mod State;

/// Internal utility sub-modules for persistence, scanning, etc.
pub mod Internal;

/// Data Transfer Objects for serialization.
pub mod DTO;

// --- Re-exports for backward compatibility ---

/// Re-export the main ApplicationState struct and helpers for backward
/// compatibility
pub use State::ApplicationState::{ApplicationState, MapLockError, MapLockErrorWithRecovery, StateOperationResult};
pub use State::{ConfigurationState, ExtensionState, FeatureState, UIState, WorkspaceState};
pub use Internal::{ExtensionScanner, PathResolution, Persistence, Recovery, Serialization, TextProcessing};
