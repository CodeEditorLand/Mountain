// File: AppState/Resolve.rs
// Defines a helper function for resolving the filesystem path for Memento
// storage.

#![allow(non_snake_case, non_camel_case_types)]

use std::path::{Path, PathBuf};

/// Resolves the absolute path for a Memento storage file based on scope and
/// workspace ID.
///
/// # Argument
/// * `AppDataDirectory` - The base application data directory.
/// * `IsGlobalScope` - If true, resolves the global storage path; otherwise,
///   resolves the workspace storage path.
/// * `WorkspaceIdentifier` - A unique identifier for the current workspace,
///   used for non-global storage.
pub fn ResolveMementoStorageFilePath(AppDataDirectory:&Path, IsGlobalScope:bool, WorkspaceIdentifier:&str) -> PathBuf {
	let UserStorageBasePath = AppDataDirectory.join("User");
	if IsGlobalScope {
		UserStorageBasePath.join("globalStorage.json")
	} else {
		// Sanitize the workspace identifier to make it a valid directory name.
		let Segment = WorkspaceIdentifier.replace(
			|Character:char| !Character.is_alphanumeric() && Character != '-' && Character != '_',
			"_",
		);
		UserStorageBasePath.join("workspaceStorage").join(Segment).join("storage.json")
	}
}
