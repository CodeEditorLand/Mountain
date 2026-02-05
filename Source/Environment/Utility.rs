//! # Utility (Environment)
//!
//! Shared utility functions used across all Environment provider implementations
//! in [`MountainEnvironment`](crate::Environment::MountainEnvironment::MountainEnvironment).
//! These handle cross-cutting concerns: error mapping, security validation,
//! language detection, and URI conversions.
//!
//! RESPONSIBILITIES:
//! - **Error Mapping**: Convert `PoisonError` from Mutex locks to [`CommonError::StateLockPoisoned`]
//! - **Security Validation**: Enforce workspace trust and path access boundaries via [`IsPathAllowedForAccess`]
//! - **Language Detection**: Infer language identifiers from file extensions (basic mapping)
//! - **URI Conversion**: Parse VS Code `UriComponents` DTOs into Rust [`Url`] types
//!
//! SECURITY MODEL:
//! - [`IsPathAllowedForAccess`] is the primary security gate for all filesystem operations:
//!   - Requires workspace trust (`ApplicationState.IsTrusted`)
//!   - Path must be within one of the registered workspace folders
//!   - Prevents extensions from accessing arbitrary system files
//!   - Trust status is atomic and thread-safe via `AtomicBool`
//!
//! ERROR HANDLING:
//! - All functions return [`CommonError`](CommonLibrary::Error::CommonError) on failure
//! - Mutex lock poisoning is mapped to `StateLockPoisoned` with context
//! - Path access violations return `FileSystemPermissionDenied`
//! - URI parsing failures return `InvalidArgument` with descriptive reason
//!
//! PERFORMANCE:
//! - Language detection is O(1) match on file extension
//! - Path validation iterates workspace folders (O(n), but typically small n)
//! - TODO: Add caching for language detection results
//!
//! VS CODE REFERENCE:
//! - `vs/base/common/network.ts` - URI handling and conversion
//! - `vs/workbench/services/files/common/fileService.ts` - path validation patterns
//! - `vs/workbench/common/resources.ts` - workspace trust model
//!
//! TODO:
//! - Add more comprehensive language detection (from .editorconfig, shebang, etc.)
//! - Implement path normalization to prevent directory traversal attacks
//! - Add caching for language detection results (LRU cache)
//! - Consider adding symbolic link resolution with security checks
//! - Add support for custom language mappings from user settings
//! - Implement path-based permission levels (read-only, read/write per folder)
//! - Add audit logging for path access attempts (security monitoring)
//! - Consider using `path_slash` crate for cross-platform path normalization
//!
//! MODULE CONTENTS:
//! - [`MapApplicationStateLockErrorToCommonError`](Self::MapApplicationStateLockErrorToCommonError)
//! - [`MapLockErrorToCommonError`](Self::MapLockErrorToCommonError)
//! - [`DetectLanguageIdentifierFromFilePath`](Self::DetectLanguageIdentifierFromFilePath)
//! - [`IsPathAllowedForAccess`](Self::IsPathAllowedForAccess)
//! - [`GetURLFromURIComponentsDTO`](Self::GetURLFromURIComponentsDTO)

use std::{
	ffi::OsStr,
	path::Path,
	sync::{MutexGuard, PoisonError},
};

use CommonLibrary::Error::CommonError::CommonError;
use log::{error, trace};
use url::Url;

use crate::ApplicationState::ApplicationState::ApplicationState;

/// Maps a `PoisonError` from a failed `ApplicationState` Mutex lock into a
/// structured `CommonError::StateLockPoisoned`.
pub fn MapApplicationStateLockErrorToCommonError<T>(Error:PoisonError<MutexGuard<'_, T>>) -> CommonError {
	let ErrorMessage = format!("[EnvironmentUtility] Failed to lock ApplicationState section: {}", Error);

	error!("{}", ErrorMessage);

	CommonError::StateLockPoisoned { Context:ErrorMessage }
}

/// Maps a generic `PoisonError` from a failed Mutex lock into a
/// structured `CommonError::StateLockPoisoned`.
pub fn MapLockErrorToCommonError<T>(Error:PoisonError<MutexGuard<'_, T>>) -> CommonError {
	let ErrorMessage = format!("[EnvironmentUtility] Failed to lock Mutex: {}", Error);

	error!("{}", ErrorMessage);

	CommonError::StateLockPoisoned { Context:ErrorMessage }
}

/// A simple utility to detect a language identifier string from a file path's
/// extension.
pub fn DetectLanguageIdentifierFromFilePath(Path:&Path) -> String {
	match Path.extension().and_then(OsStr::to_str) {
		Some("js") | Some("mjs") | Some("cjs") => "javascript",

		Some("ts") | Some("mts") | Some("cts") => "typescript",

		Some("jsx") => "javascriptreact",

		Some("tsx") => "typescriptreact",

		Some("rs") => "rust",

		Some("md") => "markdown",

		Some("json") => "json",

		Some("html") => "html",

		Some("css") => "css",

		_ => "plaintext",
	}
	.to_string()
}

/// A critical security helper that checks if a given filesystem path is
/// allowed for access.
///
/// In this architecture, this means the path must be a descendant of one of the
/// currently open and trusted workspace folders. This prevents extensions from
/// performing arbitrary filesystem operations outside the user's intended
/// scope.
pub fn IsPathAllowedForAccess(ApplicationState:&ApplicationState, PathToCheck:&Path) -> Result<(), CommonError> {
	trace!("[EnvironmentSecurity] Verifying path: {}", PathToCheck.display());

	if !ApplicationState.IsTrusted.load(std::sync::atomic::Ordering::Relaxed) {
		return Err(CommonError::FileSystemPermissionDenied {
			Path:PathToCheck.to_path_buf(),

			Reason:"Workspace is not trusted. File access is denied.".to_string(),
		});
	}

	let FoldersGuard = ApplicationState
		.WorkspaceFolders
		.lock()
		.map_err(MapApplicationStateLockErrorToCommonError)?;

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
			Path:PathToCheck.to_path_buf(),

			Reason:"Path is outside of the registered workspace folders.".to_string(),
		})
	}
}

/// Helper to get a `Url` from a `serde_json::Value` which is expected to be a
/// `UriComponents` DTO from VS Code.
pub fn GetURLFromURIComponentsDTO(URIDTO:&serde_json::Value) -> Result<Url, CommonError> {
	// VS Code's UriComponents DTO often serializes to an object with a path,

	// scheme, etc., but also includes a pre-formatted 'external' string version.
	let URIString = URIDTO.get("external").and_then(serde_json::Value::as_str).ok_or_else(|| {
		CommonError::InvalidArgument {
			ArgumentName:"URIDTO".to_string(),

			Reason:"Missing 'external' string field in UriComponents DTO".to_string(),
		}
	})?;

	Url::parse(URIString).map_err(|Error| {
		CommonError::InvalidArgument {
			ArgumentName:"URIDTO.external".to_string(),

			Reason:format!("Failed to parse URI string '{}': {}", URIString, Error),
		}
	})
}
