// @module Utility (Environment)
// @description Contains shared helper functions used by the Environment
// provider implementations and their corresponding Handler.

#![allow(non_snake_case)]

use std::{
	ffi::OsStr,
	path::{Path, PathBuf},
};

use Common::error::CommonError;
use log::{error, trace, warn};
use tauri::{AppHandle, Manager, Wry};
use url::Url;

use crate::ApplicationState::ApplicationState::ApplicationState;

// Maps a `PoisonError` from a failed `ApplicationState` Mutex lock into a
// structured `CommonError::StateLock`.
pub fn MapAppStateLockErrorToCommonError<T>(error:std::sync::PoisonError<std::sync::MutexGuard<'_, T>>) -> CommonError {
	let error_message = format!("[EnvironmentUtils] Failed to lock ApplicationState section: {}", error);
	error!("{}", error_message);
	CommonError::StateLock { Context:error_message }
}

// Maps a standard `std::io::Error` to a more specific `CommonError` variant,
// providing better context for filesystem failures.
pub fn MapIoErrorToCommonError(error:std::io::Error, path:PathBuf, operation_context:&'static str) -> CommonError {
	warn!(
		"[EnvironmentUtils] FS op '{}' on '{}' failed: {}",
		operation_context,
		path.display(),
		error
	);
	match error.kind() {
		std::io::ErrorKind::NotFound => CommonError::FileSystemNotFound(path),
		std::io::ErrorKind::PermissionDenied => CommonError::FileSystemPermissionDenied { Path:path, Reason:error.to_string() },
		std::io::ErrorKind::AlreadyExists => CommonError::FileSystemFileExists(path),
		_ => {
			CommonError::FileSystemIo {
				Path:path,
				Description:format!("Operation '{}' failed: {}", operation_context, error),
			}
		},
	}
}

// A simple utility to detect a language identifier string from a file path's
// extension.
pub fn DetectLanguageIdentifierFromFilePath(path:&Path) -> String {
	match path.extension().and_then(OsStr::to_str) {
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
pub fn DetectFileEncodingFromBytes(_content_bytes:&[u8]) -> String { "utf8".to_string() }

// A critical security helper that checks if a given filesystem path is allowed
// for access. In our architecture, this means the path must be within one of
// the currently open workspace folders.
pub async fn IsPathAllowedForFilesystemAccess(
	app_handle:&AppHandle<Wry>,
	path_to_check:&Path,
) -> Result<(), CommonError> {
	trace!("[Environment SecCheck] Verifying path: {}", path_to_check.display());

	let app_state = app_handle.state::<ApplicationState>();
	let folders_guard = app_state.WorkspaceFolders.lock().map_err(MapAppStateLockErrorToCommonError)?;

	let is_allowed = folders_guard.iter().any(|folder| {
		if let Ok(folder_path) = folder.Uri.to_file_path() {
			path_to_check.starts_with(folder_path)
		} else {
			false
		}
	});

	if is_allowed {
		Ok(())
	} else {
		Err(CommonError::FileSystemPermissionDenied {
			Path:path_to_check.to_path_buf(),
			Reason:"Path is outside of the registered workspace folders.".to_string(),
		})
	}
}

// Helper to get a `Url` from a `serde_json::Value` which is expected to be a
// `UriComponents` DTO from VS Code.
pub fn GetUrlFromUriDTO(uri_DTO:&Value) -> Result<Url, CommonError> {
	let uri_str = uri_DTO.get("external").and_then(Value::as_str).ok_or_else(|| {
		CommonError::InvalidArg {
			ArgumentName:"UriDTO".to_string(),
			Reason:"Missing 'external' field in UriComponents DTO".to_string(),
		}
	})?;
	Url::parse(uri_str).map_err(|e| {
		CommonError::InvalidArg {
			ArgumentName:"UriDTO.external".to_string(),
			Reason:format!("Failed to parse URI string '{}': {}", uri_str, e),
		}
	})
}
