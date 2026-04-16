//! # ResolveMementoStorageFilePath Module (Internal)
//!
//! ## RESPONSIBILITIES
//! Resolves the absolute path for a Memento storage file based on scope.
//! Handles both global and workspace-scoped memento paths with proper
//! sanitization.
//!
//! ## ARCHITECTURAL ROLE
//! ResolveMementoStorageFilePath is part of the **Internal::PathResolution**
//! module, resolving memento storage file paths.
//!
//! ## KEY COMPONENTS
//! - ResolveMementoStorageFilePath: Function to resolve memento paths
//!
//! ## ERROR HANDLING
//! - Sanitizes workspace identifiers to be filesystem-safe
//! - Uses alphanumeric, hyphens, and underscores only
//!
//! ## LOGGING
//! Operations are logged at appropriate levels (debug).
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Efficient path manipulation
//! - Sanitization prevents filesystem issues
//!
//! ## TODO
//! - [ ] Add path validation
//! - [ ] Implement path normalization
//! - [ ] Add cross-platform path handling

use std::path::Path;
use crate::dev_log;


/// Resolves the absolute path for a Memento storage file based on scope.
///
/// # Arguments
/// * `ApplicationDataDirectory` - Base application data directory
/// * `IsGlobalScope` - True for global storage, false for workspace storage
/// * `WorkspaceIdentifier` - Workspace identifier (ignored for global scope)
///
/// # Returns
/// PathBuf pointing to the memento storage file
///
/// # Behavior
/// - Global scope: `{AppData}/User/globalStorage.json`
/// - Workspace scope:
///   `{AppData}/User/workspaceStorage/{sanitized-id}/storage.json`
/// - Sanitizes workspace identifier (alphanumeric, hyphens, underscores only)
pub fn ResolveMementoStorageFilePath(
	ApplicationDataDirectory:&Path,
	IsGlobalScope:bool,
	WorkspaceIdentifier:&str,
) -> std::path::PathBuf {
	let user_storage_base_path = ApplicationDataDirectory.join("User");

	if IsGlobalScope {
		let path = user_storage_base_path.join("globalStorage.json");
		dev_log!("storage", 
			"[ResolveMementoStorageFilePath] Resolved global memento path: {}",
			path.display()
		);
		path
	} else {
		// Sanitize the workspace identifier to be a safe directory name
		let segment = WorkspaceIdentifier.replace(|c:char| !c.is_alphanumeric() && c != '-' && c != '_', "_");

		let path = user_storage_base_path
			.join("workspaceStorage")
			.join(&segment)
			.join("storage.json");

		dev_log!("storage", 
			"[ResolveMementoStorageFilePath] Resolved workspace memento path: {}",
			path.display()
		);

		path
	}
}
