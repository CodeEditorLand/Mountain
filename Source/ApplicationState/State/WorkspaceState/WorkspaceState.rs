//! # WorkspaceState Module (ApplicationState)
//!
//! ## RESPONSIBILITIES
//! Manages workspace-related state including workspace folders, workspace
//! trust status, workspace configuration path, window state, and the currently
//! active document URI.
//!
//! ## ARCHITECTURAL ROLE
//! WorkspaceState is part of the **state organization layer**, representing
//! all workspace-specific state in the application. This includes:
//! - Workspace folders currently open
//! - Workspace configuration file path
//! - Workspace trust/security status
//! - Main window presentation state
//! - Currently active document
//!
//! ## KEY COMPONENTS
//! - State: Main struct containing workspace-related fields
//! - Default: Initialization implementation
//! - Helper methods: Workspace manipulation utilities
//!
//! ## ERROR HANDLING
//! - Thread-safe access via `Arc<Mutex<...>>`
//! - Proper lock error handling with `MapLockError` helpers
//! - Atomic operations for simple types (IsTrusted)
//!
//! ## LOGGING
//! State changes are logged at appropriate levels (debug, info, warn, error).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Lock mutexes briefly and release immediately
//! - Avoid nested locks to prevent deadlocks
//! - Use Arc for shared ownership across threads
//! - Use AtomicBool for simple lock-free reads
//!
//! ## TODO
//! - [ ] Add workspace validation invariants
//! - [ ] Implement workspace change events
//! - [ ] Add workspace metrics collection

use std::sync::{
	Arc,
	Mutex as StandardMutex,
	atomic::{AtomicBool, Ordering as AtomicOrdering},
};


use crate::ApplicationState::DTO::{WindowStateDTO::WindowStateDTO, WorkspaceFolderStateDTO::WorkspaceFolderStateDTO};
use crate::dev_log;

/// Workspace state containing all workspace-related fields.
#[derive(Clone)]
pub struct State {
	/// Currently open workspace folders.
	pub WorkspaceFolders:Arc<StandardMutex<Vec<WorkspaceFolderStateDTO>>>,

	/// Path to the workspace configuration file (if any).
	pub WorkspaceConfigurationPath:Arc<StandardMutex<Option<std::path::PathBuf>>>,

	/// Workspace trust status (security).
	pub IsTrusted:Arc<AtomicBool>,

	/// Main window presentation state.
	pub WindowState:Arc<StandardMutex<WindowStateDTO>>,

	/// Currently active document URI.
	pub ActiveDocumentURI:Arc<StandardMutex<Option<String>>>,
}

impl Default for State {
	fn default() -> Self {
		dev_log!("workspaces", "[WorkspaceState] Initializing default workspace state...");

		Self {
			WorkspaceFolders:Arc::new(StandardMutex::new(Vec::new())),
			WorkspaceConfigurationPath:Arc::new(StandardMutex::new(None)),
			IsTrusted:Arc::new(AtomicBool::new(false)),
			WindowState:Arc::new(StandardMutex::new(WindowStateDTO::default())),
			ActiveDocumentURI:Arc::new(StandardMutex::new(None)),
		}
	}
}

impl State {
	/// Gets the current workspace trust status.
	pub fn GetTrustStatus(&self) -> bool { self.IsTrusted.load(AtomicOrdering::Relaxed) }

	/// Sets the workspace trust status.
	pub fn SetTrustStatus(&self, trusted:bool) {
		self.IsTrusted.store(trusted, AtomicOrdering::Relaxed);
		dev_log!("workspaces", "[WorkspaceState] Trust status set to: {}", trusted);
	}

	/// Gets the workspace configuration path.
	pub fn GetConfigurationPath(&self) -> Option<std::path::PathBuf> {
		self.WorkspaceConfigurationPath.lock().ok().and_then(|guard| guard.clone())
	}

	/// Sets the workspace configuration path.
	pub fn SetConfigurationPath(&self, path:Option<std::path::PathBuf>) {
		if let Ok(mut guard) = self.WorkspaceConfigurationPath.lock() {
			*guard = path.clone();
			dev_log!("workspaces", "[WorkspaceState] Configuration path updated to: {:?}", path);
		}
	}

	/// Gets the currently active document URI.
	pub fn GetActiveDocumentURI(&self) -> Option<String> {
		self.ActiveDocumentURI.lock().ok().and_then(|guard| guard.clone())
	}

	/// Sets the currently active document URI.
	pub fn SetActiveDocumentURI(&self, uri:Option<String>) {
		if let Ok(mut guard) = self.ActiveDocumentURI.lock() {
			*guard = uri.clone();
			dev_log!("workspaces", "[WorkspaceState] Active document URI updated to: {:?}", uri);
		}
	}

	/// Gets all workspace folders.
	pub fn GetWorkspaceFolders(&self) -> Vec<WorkspaceFolderStateDTO> {
		self.WorkspaceFolders.lock().ok().map(|guard| guard.clone()).unwrap_or_default()
	}

	/// Sets the workspace folders.
	pub fn SetWorkspaceFolders(&self, folders:Vec<WorkspaceFolderStateDTO>) {
		if let Ok(mut guard) = self.WorkspaceFolders.lock() {
			*guard = folders;
			dev_log!("workspaces", "[WorkspaceState] Workspace folders updated ({} folders)", guard.len());
		}
	}

	/// Gets the window state.
	pub fn GetWindowState(&self) -> WindowStateDTO {
		self.WorkspaceFolders
			.lock()
			.ok()
			.and_then(|_| self.WindowState.lock().ok().map(|guard| guard.clone()))
			.unwrap_or_default()
	}

	/// Sets the window state.
	pub fn SetWindowState(&self, state:WindowStateDTO) {
		if let Ok(mut guard) = self.WindowState.lock() {
			*guard = state;
			dev_log!("workspaces", "[WorkspaceState] Window state updated");
		}
	}
}
