// ---------------------------------------------------------------------------------------------
// Mountain Environment - Utility Functions (environment/utils.rs)
// --------------------------------------------------------------------------------------------
// This module contains shared helper functions used by various provider
// implementations within the MountainEnvironment. These utilities handle tasks
// such as error mapping, path manipulation, security checks, and text
// processing.
// --------------------------------------------------------------------------------------------

use std::{
	ffi::OsStr,
	path::{Path, PathBuf},
	sync::MutexGuard as StdMutexGuard, // Renamed to avoid conflict
};

use Land_Common::errors::CommonError;
use log::{error, trace, warn};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime}; // Added AppHandle, Manager, Runtime
use url::Url;

use crate::app_state::AppState; // For is_path_allowed_for_filesystem_access

// --- Error Mapping Helpers ---

/// Maps a `PoisonError` from `AppState` Mutex locks to a
/// `CommonError::StateLock`.
pub(super) fn map_app_state_lock_error_to_common_error<T>(
	e:std::sync::PoisonError<StdMutexGuard<'_, T>>,
) -> CommonError {
	let err_msg = format!("[Env Utils] Failed to lock AppState section: {}", e);
	error!("{}", err_msg); // Log the specific lock error
	CommonError::StateLock(err_msg)
}

/// Maps `std::io::Error` to a `CommonError` variant, providing context.
pub(super) fn map_io_error_to_common_error(
	e:std::io::Error,
	path:PathBuf,           // Path involved in the IO operation
	operation:&'static str, // Description of the FS operation (e.g., "read", "write")
) -> CommonError {
	warn!(
		"[Env Utils IOError] FS op '{}' on '{}' failed: {}",
		operation,
		path.display(),
		e
	);
	match e.kind() {
		std::io::ErrorKind::NotFound => CommonError::FsNotFound(path),
		std::io::ErrorKind::PermissionDenied => CommonError::FsPermissionDenied { path, reason:e.to_string() },
		std::io::ErrorKind::AlreadyExists => CommonError::FsFileExists(path),
		std::io::ErrorKind::IsADirectory => CommonError::FsIsADirectory(path),
		std::io::ErrorKind::NotADirectory => CommonError::FsNotADirectory(path),
		_ => {
			match operation {
				"read" | "read_doc_open" => CommonError::FsRead { path, description:e.to_string() },
				"write" | "write_doc_save" | "write_doc_save_as" | "create_file" => {
					CommonError::FsWrite { path, description:e.to_string() }
				},
				"stat" | "copy_stat" | "delete_stat_check" | "rename_target_stat" => {
					CommonError::FsStat { path, description:e.to_string() }
				},
				"mkdir" | "mkdir_all" | "mkdir_parent" | "mkdir_parent_rename" | "mkdir_parent_copy" => {
					CommonError::FsMkdir { path, description:e.to_string() }
				},
				"delete" => CommonError::FsDelete { path, description:e.to_string() },
				"rename" => CommonError::FsRename { source:path, target:PathBuf::new(), description:e.to_string() },
				"copy" => CommonError::FsCopy { source:path, target:PathBuf::new(), description:e.to_string() },
				"readdir" | "readdir_next" => CommonError::FsReadDir { path, description:e.to_string() },
				_ => {
					CommonError::Unknown(format!("Unknown FS Op '{}' on '{}' failed: {}", operation, path.display(), e))
				},
			}
		},
	}
}

// --- Text Processing and Path Utilities ---

/// Detects a language ID string from a file path's extension.
pub(super) fn detect_language_id_from_file_path(path:&Path) -> String {
	match path.extension().and_then(OsStr::to_str) {
		Some("js") | Some("mjs") | Some("cjs") => "javascript",
		Some("jsx") => "javascriptreact",
		Some("ts") => "typescript",
		Some("tsx") => "typescriptreact",
		Some("json") => "json",
		Some("jsonc") => "jsonc",
		Some("html") | Some("htm") => "html",
		Some("css") => "css",
		Some("scss") | Some("sass") => "scss",
		Some("less") => "less",
		Some("md") | Some("markdown") => "markdown",
		Some("rs") => "rust",
		Some("py") => "python",
		Some("sh") | Some("bash") | Some("zsh") => "shellscript",
		Some("yaml") | Some("yml") => "yaml",
		Some("toml") => "toml",
		Some("xml") => "xml",
		Some("java") => "java",
		Some("go") => "go",
		Some("c") | Some("h") => "c",
		Some("cpp") | Some("hpp") | Some("cxx") | Some("hxx") => "cpp",
		Some("cs") => "csharp",
		Some("rb") => "ruby",
		Some("php") => "php",
		Some("swift") => "swift",
		Some("kt") | Some("kts") => "kotlin",
		Some("dart") => "dart",
		Some("lua") => "lua",
		Some("sql") => "sql",
		Some("ps1") => "powershell",
		Some("bat") | Some("cmd") => "bat",
		Some("log") => "log",
		_ => "plaintext",
	}
	.to_string()
}

/// Detects file encoding from byte content. Simplified for MVP.
pub(super) fn detect_file_encoding_from_bytes(_content_bytes:&[u8]) -> String {
	// TODO: Implement robust encoding detection (e.g., chardetng, encoding_rs, BOM
	// check).
	"utf8".to_string()
}

/// Security helper to check if a given filesystem path is allowed for access.
pub(super) async fn is_path_allowed_for_filesystem_access<R:TauriRuntime>(
	app_handle:&AppHandle<R>, // Pass AppHandle to get AppState and PathResolver
	path_to_check:&Path,
) -> Result<(), CommonError> {
	trace!("[EnvUtils SecCheck] Verifying path: {}", path_to_check.display());
	let path_to_check_owned = path_to_check.to_path_buf();

	let canonical_path_result = tokio::task::spawn_blocking(move || -> Result<PathBuf, std::io::Error> {
		match std::fs::canonicalize(&path_to_check_owned) {
			Ok(p) => Ok(p),
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
				path_to_check_owned
					.parent()
					.map_or_else(
						|| {
							Err(std::io::Error::new(
								std::io::ErrorKind::InvalidInput,
								"Path has no parent for canonicalization fallback.",
							))
						},
						std::fs::canonicalize,
					)
					.map(|cp| cp.join(path_to_check_owned.file_name().unwrap_or_else(|| std::ffi::OsStr::new(""))))
			},
			Err(e) => Err(e),
		}
	})
	.await;

	let canonical_path_to_check = match canonical_path_result {
		Ok(Ok(p)) => p,
		Ok(Err(io_err)) => {
			return Err(CommonError::FsPermissionDenied(
				path_to_check.to_path_buf(),
				format!("Path canonicalization failed: {}. Path: '{}'", io_err, path_to_check.display()),
			));
		},
		Err(join_err) => {
			return Err(CommonError::FsPermissionDenied(
				path_to_check.to_path_buf(),
				format!(
					"Task join error during canonicalization: {}. Path: '{}'",
					join_err,
					path_to_check.display()
				),
			));
		},
	};
	trace!(
		"[EnvUtils SecCheck] Canonical path for '{}': '{}'",
		path_to_check.display(),
		canonical_path_to_check.display()
	);

	let mut allowed_root_paths:Vec<PathBuf> = Vec::new();
	let app_state = app_handle.state::<AppState>(); // Get AppState via AppHandle

	let folders_guard = app_state
		.workspace_folders
		.lock()
		.map_err(map_app_state_lock_error_to_common_error)?;
	for folder in folders_guard.iter() {
		if folder.uri.scheme() == "file" {
			// This std::fs::canonicalize is blocking. Consider alternatives if this becomes
			// a bottleneck.
			if let Ok(cfp) = std::fs::canonicalize(PathBuf::from(folder.uri.path())) {
				allowed_root_paths.push(cfp);
			} else {
				warn!("[EnvUtils SecCheck] Failed to canonicalize workspace folder: {}", folder.uri);
			}
		}
	}
	drop(folders_guard);

	let path_resolver = app_handle.path_resolver();
	for dir_opt in [
		path_resolver.app_config_dir(),
		path_resolver.app_data_dir(),
		path_resolver.app_log_dir(),
	] {
		if let Some(dp) = dir_opt {
			if let Ok(cap) = std::fs::canonicalize(&dp) {
				allowed_root_paths.push(cap);
			} else {
				warn!("[EnvUtils SecCheck] Failed to canonicalize app system dir: {}", dp.display());
			}
		}
	}

	if let Ok(cgm_path) = std::fs::canonicalize(&app_state.global_memento_path) {
		allowed_root_paths.push(cgm_path.clone());
		if let Some(p_parent) = cgm_path.parent() {
			if let Ok(p) = std::fs::canonicalize(p_parent) {
				allowed_root_paths.push(p);
			}
		}
	}
	if let Some(ref ws_m_path_opt) = *app_state
		.workspace_memento_path
		.lock()
		.map_err(map_app_state_lock_error_to_common_error)?
	{
		if let Some(ref ws_m_path) = ws_m_path_opt {
			if let Ok(cwm_path) = std::fs::canonicalize(ws_m_path) {
				allowed_root_paths.push(cwm_path.clone());
				if let Some(p_parent) = cwm_path.parent() {
					if let Ok(p) = std::fs::canonicalize(p_parent) {
						allowed_root_paths.push(p);
					}
				}
			}
		}
	}

	let is_allowed = allowed_root_paths
		.iter()
		.any(|root| canonical_path_to_check == *root || canonical_path_to_check.starts_with(root));
	if is_allowed {
		trace!("[EnvUtils SecCheck] ALLOWED: '{}'", path_to_check.display());
		Ok(())
	} else {
		warn!(
			"[EnvUtils SecCheck] DENIED: '{}' (canonical: '{}'). Not in roots: {:?}",
			path_to_check.display(),
			canonical_path_to_check.display(),
			allowed_root_paths
		);
		Err(CommonError::FsPermissionDenied(
			path_to_check.to_path_buf(),
			"Path outside allowed workspace/app data folders.".to_string(),
		))
	}
}
