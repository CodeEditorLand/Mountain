//! # ApplicationState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Central state management for the Mountain application, aggregating all
//! state modules into a single source of truth.
//!
//! ## ARCHITECTURAL ROLE
//! The ApplicationState is the **state container** that aggregates all
//! domain-specific state modules and provides thread-safe access.
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
//! ## KEY COMPONENTS
//! - Workspace: Workspace folders, trust, active document
//! - Configuration: Configuration, memento storage
//! - Extension: Extension registry, providers, scanned extensions
//! - Feature: Diagnostics, documents, terminals, webviews, etc.
//! - UI: Pending UI requests
//!
//! ## ERROR HANDLING
//! All state operations use `Arc<Mutex<...>>` for thread-safety with proper
//! error handling via `MapLockError` helpers.
//!
//! ## LOGGING
//! State changes are logged at appropriate levels (debug, info, warn, error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Lock mutexes briefly and release immediately
//! - Avoid nested locks to prevent deadlocks
//! - Use Arc for shared ownership across threads
//!
//! ## TODO
//! - [ ] Add state validation invariants
//! - [ ] Implement state metrics collection
//! - [ ] Add state diffing for debugging

use std::sync::{Arc, Mutex as StandardMutex, PoisonError};

use CommonLibrary::Error::CommonError::CommonError;
use log::debug;

use super::{
	WorkspaceState::State as WorkspaceState,
	ConfigurationState::State as ConfigurationState,
	ExtensionState::State::State as ExtensionState,
	FeatureState::State::State as FeatureState,
	UIState::State as UIState,
};
use crate::Environment::TestProvider::TestProviderState;

/// The central, shared, thread-safe state for the entire Mountain application.
#[derive(Clone)]
pub struct ApplicationState {
	/// Workspace state containing workspace folders, trust, and active document.
	pub Workspace: WorkspaceState,

	/// Configuration and storage state.
	pub Configuration: ConfigurationState,

	/// Extension management state.
	pub Extension: ExtensionState,

	/// Feature-specific state.
	pub Feature: FeatureState,

	/// User interface request state.
	pub UI: UIState,

	/// Test provider state.
	pub TestProviderState: Arc<tokio::sync::RwLock<TestProviderState>>,

	/// Memento storage paths.
	pub GlobalMementoPath: std::path::PathBuf,
	pub WorkspaceMementoPath: Arc<StandardMutex<Option<std::path::PathBuf>>>,
}

impl Default for ApplicationState {
	fn default() -> Self {
		debug!("[ApplicationState] Initializing default application state...");

		Self {
			Workspace: Default::default(),
			Configuration: Default::default(),
			Extension: Default::default(),
			Feature: Default::default(),
			UI: Default::default(),
			TestProviderState: Arc::new(tokio::sync::RwLock::new(
				TestProviderState::new(),
			)),
			GlobalMementoPath: Default::default(),
			WorkspaceMementoPath: Arc::new(StandardMutex::new(None)),
		}
	}
}

impl ApplicationState {
	/// Gets the next available unique identifier for a provider registration.
	pub fn GetNextProviderHandle(&self) -> u32 { self.Extension.GetNextProviderHandle() }

	/// Gets the next available unique identifier for a terminal instance.
	pub fn GetNextTerminalIdentifier(&self) -> u64 {
		self.Feature.GetNextTerminalIdentifier()
	}

	/// Gets the next available unique identifier for an SCM provider.
	pub fn GetNextSourceControlManagementProviderHandle(&self) -> u32 {
		self.Feature.GetNextSourceControlManagementProviderHandle()
	}

	/// Gets the workspace identifier for the current application instance.
	/// This is used to differentiate between different workspace instances
	/// when running multiple instances of the application.
	pub fn GetWorkspaceIdentifier(&self) -> Result<String, CommonError> {
		// For now, generate a simple identifier based on the current timestamp
		// In a proper implementation, this would be stored and persisted
		use std::time::{SystemTime, UNIX_EPOCH};
		
		let timestamp = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.map_err(|e| CommonError::Unknown { Description: format!("Failed to get system time: {}", e) })?;
		
		Ok(format!("workspace-{:x}", timestamp.as_millis()))
	}
}

/// A helper to map a mutex poison error into a CommonError.
pub fn MapLockError<T>(Error: PoisonError<T>) -> CommonError {
	CommonError::StateLockPoisoned { Context: Error.to_string() }
}

/// A helper to map a mutex poison error with recovery attempt.
pub fn MapLockErrorWithRecovery<T>(Error: PoisonError<T>, RecoveryContext: &str) -> CommonError {
	log::warn!(
		"[ApplicationState] Attempting recovery from poisoned lock in context: {}",
		RecoveryContext
	);
	CommonError::StateLockPoisoned {
		Context: format!("{} - Recovery attempted: {}", Error.to_string(), RecoveryContext),
	}
}

/// Error handling result with recovery information.
#[derive(Debug)]
pub struct StateOperationResult<T> {
	pub result: Result<T, CommonError>,
	pub recovery_attempted: bool,
	pub recovery_successful: bool,
}
