//! # Environment Utility
//!
//! Contains shared helper functions used by the `MountainEnvironment` provider
//! implementations for tasks like error mapping, path manipulation, and
//! security checks.

#![allow(non_snake_case, non_camel_case_types)]

use std::{
	ffi::OsStr,
	path::Path,
	sync::{MutexGuard, PoisonError},
};

use Common::Error::CommonError::CommonError;
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
		.WorkSpaceFolders
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
