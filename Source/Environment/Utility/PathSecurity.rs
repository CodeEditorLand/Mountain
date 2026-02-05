//! # Path Security Utilities
//!
//! Functions for validating filesystem access and enforcing workspace trust.

use std::path::Path;

use CommonLibrary::Error::CommonError::CommonError;
use log::trace;

use crate::ApplicationState::ApplicationState::ApplicationState;

/// A critical security helper that checks if a given filesystem path is
/// allowed for access.
///
/// In this architecture, this means the path must be a descendant of one of the
/// currently open and trusted workspace folders. This prevents extensions from
/// performing arbitrary filesystem operations outside the user's intended
/// scope.
pub fn IsPathAllowedForAccess(ApplicationState: &ApplicationState, PathToCheck: &Path) -> Result<(), CommonError> {
	trace!("[EnvironmentSecurity] Verifying path: {}", PathToCheck.display());

	if !ApplicationState.IsTrusted.load(std::sync::atomic::Ordering::Relaxed) {
		return Err(CommonError::FileSystemPermissionDenied {
			Path: PathToCheck.to_path_buf(),
			Reason: "Workspace is not trusted. File access is denied.".to_string(),
		});
	}

	let FoldersGuard = ApplicationState
		.WorkspaceFolders
		.lock()
		.map_err(super::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;

	if FoldersGuard.is_empty() {
		// Allow access if no folder is open, as operations are likely on user-chosen
		// files. A stricter model could deny this.
		return Ok(());
	}

	let IsAllowed = FoldersGuard.iter().any(|Folder| {
		match Folder.URI.to_file_path() {
			Ok(FolderPath) => PathToCheck.starts_with(FolderPath),
			Err(_) => false,
		}
	});

	if IsAllowed {
		Ok(())
	} else {
		Err(CommonError::FileSystemPermissionDenied {
			Path: PathToCheck.to_path_buf(),
			Reason: "Path is outside of the registered workspace folders.".to_string(),
		})
	}
}
