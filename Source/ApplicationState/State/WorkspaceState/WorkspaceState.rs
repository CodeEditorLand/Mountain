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

use std::{
	sync::{
		Arc,
		atomic::{AtomicBool, Ordering as AtomicOrdering},
	},
};

use parking_lot::Mutex;

use crate::{
	ApplicationState::DTO::{WindowStateDTO::WindowStateDTO, WorkspaceFolderStateDTO::WorkspaceFolderStateDTO},
	dev_log,
};

/// Workspace state containing all workspace-related fields.
#[derive(Clone)]
pub struct State {
	/// Currently open workspace folders.
	pub WorkspaceFolders:Arc<Mutex<Vec<WorkspaceFolderStateDTO>>>,

	/// Path to the workspace configuration file (if any).
	pub WorkspaceConfigurationPath:Arc<Mutex<Option<std::path::PathBuf>>>,

	/// Workspace trust status (security).
	pub IsTrusted:Arc<AtomicBool>,

	/// Main window presentation state.
	pub WindowState:Arc<Mutex<WindowStateDTO>>,

	/// Currently active document URI.
	pub ActiveDocumentURI:Arc<Mutex<Option<String>>>,
}

impl Default for State {
	fn default() -> Self {
		dev_log!("workspaces", "[WorkspaceState] Initializing default workspace state...");

		Self {
			WorkspaceFolders:Arc::new(Mutex::new(Vec::new())),

			WorkspaceConfigurationPath:Arc::new(Mutex::new(None)),

			IsTrusted:Arc::new(AtomicBool::new(false)),

			WindowState:Arc::new(Mutex::new(WindowStateDTO::default())),

			ActiveDocumentURI:Arc::new(Mutex::new(None)),
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
		self.WorkspaceConfigurationPath.lock().clone()
	}

	/// Sets the workspace configuration path.
	pub fn SetConfigurationPath(&self, path:Option<std::path::PathBuf>) {
		let mut guard = self.WorkspaceConfigurationPath.lock();
		*guard = path.clone();
		dev_log!("workspaces", "[WorkspaceState] Configuration path updated to: {:?}", path);
	}

	/// Gets the currently active document URI.
	pub fn GetActiveDocumentURI(&self) -> Option<String> {
		self.ActiveDocumentURI.lock().clone()
	}

	/// Sets the currently active document URI.
	pub fn SetActiveDocumentURI(&self, uri:Option<String>) {
		let mut guard = self.ActiveDocumentURI.lock();
		*guard = uri.clone();
		dev_log!("workspaces", "[WorkspaceState] Active document URI updated to: {:?}", uri);
	}

	/// Gets all workspace folders.
	pub fn GetWorkspaceFolders(&self) -> Vec<WorkspaceFolderStateDTO> {
		self.WorkspaceFolders.lock().clone()
	}

	/// Sets the workspace folders.
	pub fn SetWorkspaceFolders(&self, folders:Vec<WorkspaceFolderStateDTO>) {
		let mut guard = self.WorkspaceFolders.lock();
		*guard = folders;
		dev_log!(
			"workspaces",
			"[WorkspaceState] Workspace folders updated ({} folders)",
			guard.len()
		);
	}

	/// Atomically replace the workspace folders and return the (added, removed)
	/// delta. `added` contains every folder present in the new list but not the
	/// old one; `removed` contains every folder present in the old list but not
	/// the new. Comparison is by URI, so re-indexing does not produce spurious
	/// add/remove pairs.
	///
	/// Callers use the delta to drive downstream events such as
	/// `$deltaWorkspaceFolders` (Cocoon) and `onDidChangeWorkspaceFolders`
	/// listeners inside extensions.
	pub fn SetWorkspaceFoldersReturnDelta(
		&self,

		folders:Vec<WorkspaceFolderStateDTO>,
	) -> (Vec<WorkspaceFolderStateDTO>, Vec<WorkspaceFolderStateDTO>) {
		let mut guard = self.WorkspaceFolders.lock();
		let Old = guard.clone();

		let OldUris:std::collections::HashSet<String> = Old.iter().map(|F| F.URI.to_string()).collect();

		let NewUris:std::collections::HashSet<String> = folders.iter().map(|F| F.URI.to_string()).collect();

		let Added:Vec<WorkspaceFolderStateDTO> = folders
			.iter()
			.filter(|F| !OldUris.contains(&F.URI.to_string()))
			.cloned()
			.collect();

		let Removed:Vec<WorkspaceFolderStateDTO> =
			Old.iter().filter(|F| !NewUris.contains(&F.URI.to_string())).cloned().collect();

		*guard = folders;
		dev_log!(
			"workspaces",
			"[WorkspaceState] Workspace folders updated ({} folders, +{} -{})",
			guard.len(),
			Added.len(),
			Removed.len()
		);

		(Added, Removed)
	}

	/// Gets the window state.
	pub fn GetWindowState(&self) -> WindowStateDTO {
		self.WindowState.lock().clone()
	}

	/// Sets the window state.
	pub fn SetWindowState(&self, state:WindowStateDTO) {
		let mut guard = self.WindowState.lock();
		*guard = state;
		dev_log!("workspaces", "[WorkspaceState] Window state updated");
	}
}