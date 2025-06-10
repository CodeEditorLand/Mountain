use std::{
	ffi::OsStr,
	path::{Path, PathBuf},
};

use Common::error::CommonError;
use log::{error, trace, warn};
use tauri::{ApplicationHandle, Manager, Wry};

// @module Utils (Environment)
// @description Contains shared helper functions used by the environment
// provider implementations and their corresponding Handler.
use crate::ApplicationState::ApplicationState::ApplicationState;

// Maps a `PoisonError` from a failed `ApplicationState` Mutex lock into a
// structured `CommonError::StateLock`.
pub fn MapAppStateLockErrorToCommonError<T>(Error:std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> CommonError {
	let ErrorMessage = format!("[EnvironmentUtils] Failed to lock ApplicationState section: {}", Error);
	error!("{}", ErrorMessage);
	CommonError::StateLock { Context:ErrorMessage }
}

// Maps a standard `std::io::Error` to a more specific `CommonError` variant,
// providing better context for filesystem failures.
pub fn MapIoErrorToCommonError(Error:std::io::Error, Path:PathBuf, OperationContext:&'static str) -> CommonError {
	warn!(
		"[EnvironmentUtils] FS op '{}' on '{}' failed: {}",
		OperationContext,
		Path.display(),
		Error
	);
	match Error.kind() {
		std::io::ErrorKind::NotFound => CommonError::FsNotFound(Path),
		std::io::ErrorKind::PermissionDenied => CommonError::FsPermissionDenied { Path, Reason:Error.to_string() },
		std::io::ErrorKind::AlreadyExists => CommonError::FsFileExists(Path),
		_ => {
			CommonError::FsIo {
				Path,
				Description:format!("Operation '{}' failed: {}", OperationContext, Error.to_string()),
			}
		},
	}
}

// A simple utility to detect a language identifier string from a file path's
// extension.
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

// A stub for a function that would detect file encoding from byte content.
// A real implementation would inspect for a Byte Order Mark (BOM) or use
// heuristics.
pub fn DetectFileEncodingFromBytes(_ContentBytes:&[u8]) -> String { "utf8".to_string() }

// A critical security helper that checks if a given filesystem path is allowed
// for access. In our architecture, this means the path must be within one of
// the currently open workspace folders.
pub async fn IsPathAllowedForFilesystemAccess(ApplicationHandle:&ApplicationHandle<Wry>, PathToCheck:&Path) -> Result<(), CommonError> {
	trace!("[Environment SecCheck] Verifying path: {}", PathToCheck.display());

	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	let FoldersGuard = AppStateInstance
		.WorkspaceFolders
		.lock()
		.map_err(MapAppStateLockErrorToCommonError)?;

	let IsAllowed = FoldersGuard.iter().any(|Folder| {
		if let Ok(FolderPath) = Folder.Uri.to_file_path() {
			PathToCheck.starts_with(FolderPath)
		} else {
			false
		}
	});

	if IsAllowed {
		Ok(())
	} else {
		Err(CommonError::FsPermissionDenied {
			Path:PathToCheck.to_path_buf(),
			Reason:"Path is outside of the registered workspace folders.".to_string(),
		})
	}
}
