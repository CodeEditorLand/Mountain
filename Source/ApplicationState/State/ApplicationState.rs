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

use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use CommonLibrary::Error::CommonError::CommonError;

use super::{
	ConfigurationState::ConfigurationState::State as ConfigurationState,
	ExtensionState::State::State as ExtensionState,
	FeatureState::State::State as FeatureState,
	UIState::UIState::State as UIState,
	WorkspaceState::WorkspaceState::State as WorkspaceState,
};

use crate::{Environment::TestProvider::TestProviderState::Struct as TestProviderState, dev_log};

/// The central, shared, thread-safe state for the entire Mountain application.
pub type SharedApplicationState = Arc<ApplicationState>;

pub struct ApplicationState {

	/// Workspace state containing workspace folders, trust, and active
	/// document.
	pub Workspace:WorkspaceState,

	/// Configuration and storage state.
	pub Configuration:ConfigurationState,

	/// Extension management state.
	pub Extension:ExtensionState,

	/// Feature-specific state.
	pub Feature:FeatureState,

	/// User interface request state.
	pub UI:UIState,

	/// Test provider state.
	pub TestProviderState:Arc<RwLock<TestProviderState>>,

	/// Memento storage paths.
	pub GlobalMementoPath:Arc<Mutex<std::path::PathBuf>>,

	pub WorkspaceMementoPath:Arc<Mutex<Option<std::path::PathBuf>>>,
}

impl Default for ApplicationState {

	fn default() -> Self {
		dev_log!("lifecycle", "[ApplicationState] Initializing default application state...");

		Self {
			Workspace:Default::default(),

			Configuration:Default::default(),

			Extension:Default::default(),

			Feature:Default::default(),

			UI:Default::default(),

			TestProviderState:Arc::new(RwLock::new(TestProviderState::new())),

			GlobalMementoPath:Arc::new(Mutex::new(Default::default())),

			WorkspaceMementoPath:Arc::new(Mutex::new(None)),
		}
	}
}

impl ApplicationState {

	/// Gets the next available unique identifier for a provider registration.
	pub fn GetNextProviderHandle(&self) -> u32 { self.Extension.GetNextProviderHandle() }

	/// Gets the next available unique identifier for a terminal instance.
	pub fn GetNextTerminalIdentifier(&self) -> u64 { self.Feature.GetNextTerminalIdentifier() }

	/// Gets the next available unique identifier for an SCM provider.
	pub fn GetNextSourceControlManagementProviderHandle(&self) -> u32 {
		self.Feature.GetNextSourceControlManagementProviderHandle()
	}

	/// Gets a stable identifier for the current workspace instance.
	/// Derived from the workspace configuration path or the first
	/// workspace folder URI so it remains constant across callers for
	/// the same workspace, enabling deduplication in recently-opened
	/// lists, per-workspace storage paths, and window-title derivation.
	pub fn GetWorkspaceIdentifier(&self) -> Result<String, CommonError> {
		// Prefer the configuration file path when present; otherwise hash
		// the first workspace folder URI. Falling back to a fixed sentinel
		// keeps the result deterministic across restarts with no workspace.
		let key = if let Some(Path) = self.Workspace.GetConfigurationPath() {
			Path.to_string_lossy().to_string()
		} else if let Some(First) = self.Workspace.GetWorkspaceFolders().first() {
			First.URI.to_string()
		} else {
			return Ok("NO_WORKSPACE".to_string());
		};

		use std::{
			collections::hash_map::DefaultHasher,
			hash::{Hash, Hasher},
		};

		let mut Hasher = DefaultHasher::new();

		key.hash(&mut Hasher);

		Ok(format!("{:016x}", Hasher.finish()))
	}
}
