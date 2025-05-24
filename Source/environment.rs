// ---------------------------------------------------------------------------------------------
// Mountain Environment Implementation (environment.rs)

// --------------------------------------------------------------------------------------------
// Defines `MountainEnvironment`, the concrete implementation of the abstract
// `Environment` trait from `Land_Common`. It also implements various provider
// traits (e.g., `FsReader`, `FsWriter`, `ConfigProvider`, `DocumentProvider`,

// `UiProvider`, etc.) that define the actual "native" logic for `ActionEffect`s
// executed by the `AppRuntime`.
//
// Responsibilities:
// - Implementing all provider traits defined in `Land_Common::effects`.
// - Filesystem Access (`FsReader`, `FsWriter`):
//   - Uses `tokio::fs` for asynchronous file operations.
//   - Performs security checks (e.g., path canonicalization, ensuring paths are
//     within allowed workspace or app data boundaries via `is_path_allowed`).
// - Configuration Management (`ConfigProvider`, `ConfigInspector`):
//   - Accesses `AppState.configuration` (the merged view) for reading
//     configuration values.
//   - Uses helper functions from `handlers::config` for:
//     - Writing configuration changes to the correct `settings.json` file
//       (User, Workspace, WorkspaceFolder).
//     - Performing JSON manipulation (getting/setting values at specific key
//       paths).
//     - Triggering a re-merge of all configuration sources into
//       `AppState.configuration`.
//     - Notifying Cocoon of configuration changes via Vine.
//   - (Stubbed) `ConfigInspector` for providing detailed info about config
//     values.
// - Document State Management (`DocumentProvider`):
//   - Manages `DocumentState` instances within `AppState.open_documents`.
//   - Handles opening documents from files or creating new untitled documents.
//   - Applies content changes received from Cocoon to `DocumentState` (using
//     `DocumentState::apply_document_content_changes`).
//   - Saves documents to disk, updating their dirty state.
//   - Calls notification helpers in `handlers::documents` (e.g.,

//     `notify_model_added`, `notify_model_changed`) to inform Cocoon of
//     document state changes.
// - Storage (`StorageProvider`), Secrets (`SecretsProvider`), Diagnostics
//   (`DiagnosticsManager`), Commands (`CommandExecutor`), Output Channels
//   (`OutputChannelManager`):
//   - Mostly delegates to the corresponding handler functions in `handlers::*`
//     modules, which manage the state in `AppState` and perform necessary
//     actions.
// - Language Features (`LanguageFeatureProviderRegistry`):
//   - Manages registrations of language feature providers (e.g., hover,

//     completion) from extensions. Stores these in
//     `AppState.language_providers`.
//   - Provides a way to query for active providers based on document URI,

//     language ID, and provider type, matching against registered
//     `DocumentSelector`s.
// - UI Interactions (`UiProvider`):
//   - For simple messages without return values (e.g., `showInformationMessage`
//     with no buttons), can use Tauri's native dialog API directly (spawned on
//     a blocking thread).
//   - For complex UI interactions requiring user input or choices (e.g.,

//     open/save dialogs, quick picks, input boxes, messages with buttons):
//     - Generates a unique request ID.
//     - Stores a `tokio::sync::oneshot::Sender` in
//       `AppState.pending_ui_requests` keyed by this ID.
//     - Emits a Tauri event (e.g., `sky://ui/show-open-dialog-request`) to the
//       Sky frontend, including the request ID and necessary options.
//     - Asynchronously awaits a response on the `oneshot::Receiver`.
//     - Sky handles the UI and calls back to Mountain via the
//       `sky_resolves_ui_request` Tauri command, providing the original request
//       ID and the user's input or error.
//     - `handlers::sky_ui_responses` processes this callback, finds the pending
//       `oneshot::Sender`, and sends the result back, unblocking the
//       `UiProvider` method.
//     - Includes timeout mechanisms for these asynchronous UI operations.
// - Holding an `AppHandle<Wry>` for accessing `AppState`, Tauri APIs (path
//   resolver, event emission), and window management.
// - Mapping I/O errors and other operational failures to `CommonError` types.
//
// Key Interactions:
// - Instantiated in `main.rs` and wrapped in an `Arc` for sharing; this `Arc`
//   is held by `AppRuntime`.
// - Its methods (trait implementations) are called by `AppRuntime::run` when
//   executing `ActionEffect`s.
// - Heavily relies on `AppHandle` to get `AppState` via `self.get_app_state()`.
// - Uses helper functions from various modules in `handlers::*` for specific
//   tasks (config persistence, document notifications, etc.).
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	// For Path::extension()
	ffi::OsStr,

	path::{Path, PathBuf},

	sync::{Arc, Mutex as StdMutex, MutexGuard},

	// Standard library Duration
	time::Duration as StdDuration,
};

// Common effect traits and error types from Land_Common
use Land_Common::{
	command_effects::CommandExecutor,

	config_effects::{
		ConfigInspector,

		ConfigProvider,

		ConfigurationScope,

		ConfigurationTarget,

		IConfigurationOverrides,

		InspectResultData,
	},

	diagnostics_effects::DiagnosticsManager,

	documents_effects::{DocEventParams, DocumentProvider},

	// Core Environment trait and Requires helper
	environment::{Environment, Requires},

	errors::CommonError,

	fs_effects::{FileSystemStat, FileType as CommonFileType, FsReader, FsWriter},

	ipc_effects::IpcProvider,

	language_feature_effects::{
		LanguageFeatureProviderRegistry,

		ProviderDescription,

		ProviderType as CommonProviderType,
	},

	output_effects::OutputChannelManager,

	secrets_effects::SecretsProvider,

	storage_effects::StorageProvider,

	ui_effects::{
		DialogOptions,

		InputBoxOptions,

		MessageOptions,

		MessageSeverity,

		OpenDialogOptions,

		QuickPickItem,

		QuickPickOptions,

		SaveDialogOptions,

		UiProvider,
	},

	workspace_effects::WorkspaceProvider,
};
// For async methods in traits
use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
// For DTOs used in UiProvider event payloads
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value, json};
// Tauri essentials
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window, Wry};
use tokio::{
	// Tokio's async filesystem operations
	fs,

	// For File::write_all
	io::AsyncWriteExt,

	// For UiProvider async request-response with Sky
	sync::oneshot as TokioOneshot,

	// For UI interaction timeouts
	time::{Duration as TokioDuration, timeout as tokio_timeout},
};
use url::Url;
// For generating unique request IDs for UI interactions
use uuid::Uuid;

use crate::{
	app_state::{
		// Make app_state module accessible for its DTOs
		self,

		AppState,

		// Specific DTOs from app_state
		DocumentState,

		// Renamed to avoid conflict
		LanguageProviderType as AppStateLanguageProviderType,

		MementoStorageMap,

		MergedConfigurationState,

		OutputChannelState,

		ProviderRegistration,

		WorkspaceFolderState,
	},

	// Access to various handler modules
	handlers,

	// Not directly used by Environment, but context for AppRuntime
	runtime::AppRuntime,

	// For sending notifications (though often delegated to handlers)
	vine,
};

// --- Mountain Environment Struct ---
/// Concrete implementation of the `Environment` and various provider traits.
///
/// This struct holds an `AppHandle` to interact with Tauri and access
/// `AppState`. It provides the "native" logic that backs `ActionEffect`s.
// Clone is necessary for Arc<MountainEnvironment>
#[derive(Clone)]
pub struct MountainEnvironment {
	// Wry is the default Tauri webview runtime
	app_handle:AppHandle<Wry>,
}

impl MountainEnvironment {
	/// Creates a new `MountainEnvironment`.
	pub fn new(app_handle:AppHandle<Wry>) -> Self {
		info!("[Env Init] MountainEnvironment instance created.");

		Self { app_handle }
	}

	/// Helper to get a Tauri `State` wrapper for `AppState`.
	fn get_app_state(&self) -> State<'_, AppState> { self.app_handle.state::<AppState>() }

	/// Security helper to check if a given filesystem path is allowed for
	/// access.
	///
	/// Allowed paths include:
	/// - Paths within any open workspace folder.
	/// - Paths within standard application directories (config, data, log).
	/// - Paths for global and workspace memento storage files.
	///
	/// This function performs path canonicalization to prevent traversal
	/// attacks.
	///
	/// # Arguments
	/// * `path_to_check` - The `Path` to validate.
	///
	/// # Returns
	/// * `Ok(())` if the path is allowed.
	/// * `Err(CommonError::FsPermissionDenied)` if the path is not allowed or
	///   canonicalization fails.
	async fn is_path_allowed_for_filesystem_access(&self, path_to_check:&Path) -> Result<(), CommonError> {
		trace!("[Env Security Check] Verifying path allowance for: {}", path_to_check.display());

		// Clone for async task
		let path_to_check_owned = path_to_check.to_path_buf();

		// Canonicalize the path to resolve symlinks and relative segments (.., .).
		// This is crucial for security to prevent directory traversal attacks.
		// `std::fs::canonicalize` is blocking, so run it via `spawn_blocking`.
		let canonical_path_result = tokio::task::spawn_blocking(move || -> Result<PathBuf, std::io::Error> {



			// `canonicalize` fails if the path doesn't exist. If it's a new file/dir being
			// created, canonicalize its intended parent directory and append the
			// filename/dirname.
			match std::fs::canonicalize(&path_to_check_owned) {



				Ok(p) => Ok(p),


				Err(e) if e.kind() == std::io::ErrorKind::NotFound => {



					// If path itself not found, try to canonicalize parent and join filename.
					// This allows checking permissions for creating new files/dirs.
					path_to_check_owned
						.parent()

						.map_or_else(
							|| {



								Err(std::io::Error::new(
									std::io::ErrorKind::InvalidInput,

									"Path has no parent for canonicalization fallback.",

								))

							},

							 // Canonicalize parent
							std::fs::canonicalize,

						)

						.map(|canonical_parent| {



							canonical_parent.join(
								 // Get filename
								path_to_check_owned.file_name().unwrap_or_else(|| OsStr::new("")),

							)

						})

				},


				 // Other canonicalization error
				Err(e) => Err(e),

			}

		})

		 // Wait for spawn_blocking to complete
		.await;

		let canonical_path_to_check = match canonical_path_result {
			Ok(Ok(p)) => p,

			Ok(Err(io_err)) => {
				// Error during canonicalization (e.g., parent doesn't exist, permissions)

				return Err(CommonError::FsPermissionDenied(
					path_to_check.to_path_buf(),
					format!(
						"Path canonicalization failed for security check: {}. Path: '{}'",
						io_err,
						path_to_check.display()
					),
				));
			},

			Err(join_err) => {
				// Task join error (e.g., spawn_blocking panicked)

				return Err(CommonError::FsPermissionDenied(
					path_to_check.to_path_buf(),
					format!(
						"Task join error during path canonicalization for security check: {}. Path: '{}'",
						join_err,
						path_to_check.display()
					),
				));
			},
		};

		trace!(
			"[Env Security Check] Canonical path for '{}' is '{}'",
			path_to_check.display(),
			canonical_path_to_check.display()
		);

		// Gather all allowed root paths.
		let mut allowed_root_paths:Vec<PathBuf> = Vec::new();

		let app_state = self.get_app_state();

		// 1. Workspace Folders
		let folders_guard = app_state
			.workspace_folders
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		for folder in folders_guard.iter() {
			if folder.uri.scheme() == "file" {
				// TODO: This `std::fs::canonicalize` is blocking. If `is_path_allowed` is
				// called frequently       on hot paths, consider pre-canonicalizing
				// workspace folder paths when they are set       and storing them in
				// `AppState`, or doing this canonicalization part also with `spawn_blocking`.
				//       For now, assuming it's acceptable within the FS op that called this.
				if let Ok(canonical_folder_path) = std::fs::canonicalize(PathBuf::from(folder.uri.path())) {
					allowed_root_paths.push(canonical_folder_path);
				} else {
					warn!(
						"[Env Security Check] Failed to canonicalize workspace folder URI for check: {}",
						folder.uri
					);
				}
			}
		}

		// Release lock
		drop(folders_guard);

		// 2. Standard Application Directories (config, data, log)

		let path_resolver = self.app_handle.path_resolver();

		for dir_opt in [
			path_resolver.app_config_dir(),
			path_resolver.app_data_dir(),
			path_resolver.app_log_dir(),
			// TODO: Consider adding `app_cache_dir` if used.
		] {
			if let Some(dir_path) = dir_opt {
				if let Ok(canonical_app_dir_path) = std::fs::canonicalize(&dir_path) {
					allowed_root_paths.push(canonical_app_dir_path);
				} else {
					warn!(
						"[Env Security Check] Failed to canonicalize app system directory for check: {}",
						dir_path.display()
					);
				}
			}
		}

		// 3. Memento Storage Files (and their parent directories)

		if let Ok(canonical_global_memento_path) = std::fs::canonicalize(&app_state.global_memento_path) {
			// Allow access to the file itself and its parent dir (for atomic writes via
			// temp file)

			allowed_root_paths.push(canonical_global_memento_path);

			if let Some(parent) = app_state.global_memento_path.parent() {
				if let Ok(p) = std::fs::canonicalize(parent) {
					allowed_root_paths.push(p);
				}
			}
		}

		if let Some(ref ws_memento_path_opt) = *app_state
			.workspace_memento_path
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
		{
			if let Some(ref ws_memento_path) = ws_memento_path_opt {
				if let Ok(canonical_ws_memento_path) = std::fs::canonicalize(ws_memento_path) {
					allowed_root_paths.push(canonical_ws_memento_path);

					if let Some(parent) = ws_memento_path.parent() {
						if let Ok(p) = std::fs::canonicalize(parent) {
							allowed_root_paths.push(p);
						}
					}
				}
			}
		}

		// TODO: Add extension storage paths if they are distinct and managed by
		// Mountain.

		// Check if the canonicalized path_to_check is within any of the allowed roots.
		let is_allowed = allowed_root_paths
			.iter()
			.any(|root_path| canonical_path_to_check == *root_path || canonical_path_to_check.starts_with(root_path));

		if is_allowed {
			trace!(
				"[Env Security Check] ALLOWED: Path '{}' (canonical: '{}') is within allowed roots.",
				path_to_check.display(),
				canonical_path_to_check.display()
			);

			Ok(())
		} else {
			warn!(
				"[Env Security Check] DENIED: Path '{}' (canonical: '{}') is NOT within any allowed roots. Allowed \
				 roots: {:?}",
				path_to_check.display(),
				canonical_path_to_check.display(),
				allowed_root_paths
			);

			Err(CommonError::FsPermissionDenied(
				path_to_check.to_path_buf(),
				"Path is outside allowed workspace or application data folders.".to_string(),
			))
		}
	}
}

// Implement the core Environment trait (currently empty, acts as a marker).
impl Environment for MountainEnvironment {}

// --- Helper Error/Util Functions (Module-private) ---

/// Maps a `PoisonError` from `AppState` Mutex locks to a
/// `CommonError::StateLock`.
fn map_app_state_lock_error_to_common_error<T>(e:std::sync::PoisonError<MutexGuard<'_, T>>) -> CommonError {
	let err_msg = format!("Failed to lock AppState section: {}", e);

	// Log specific lock error
	error!("[Env AppStateLockErr] {}", err_msg);

	CommonError::StateLock(err_msg)
}

/// Maps `std::io::Error` to a `CommonError` variant, providing context.
fn map_io_error_to_common_error(
	e:std::io::Error,

	// Path involved in the IO operation
	path:PathBuf,

	// Description of the FS operation (e.g., "read", "write")
	operation:&'static str,
) -> CommonError {
	warn!(
		// Log as warning because these are common operational errors
		"[Env IOError] FS operation '{}' on path '{}' failed: {}",
		operation,
		path.display(),
		e
	);

	match e.kind() {
		std::io::ErrorKind::NotFound => CommonError::FsNotFound(path),

		std::io::ErrorKind::PermissionDenied => CommonError::FsPermissionDenied(path, e.to_string()),

		std::io::ErrorKind::AlreadyExists => CommonError::FsFileExists(path),

		std::io::ErrorKind::IsADirectory => CommonError::FsIsADirectory(path),

		std::io::ErrorKind::NotADirectory => CommonError::FsNotADirectory(path),

		// Note: `DirectoryNotEmpty` is not a standard `std::io::ErrorKind`.
		//       `fs::remove_dir` returns `ErrorKind::Other` or platform-specific for this.
		//       Custom handling might be needed if `remove_dir` fails because dir not empty.
		//       For now, relying on `FsNotEmpty` to be constructed by specific logic if `remove_dir` fails this way.
		_ => {
			// More generic mapping for other IO errors based on operation type
			match operation {
				"read" | "read_doc_open" => CommonError::FsRead(path, e.to_string()),

				"write" | "write_doc_save" | "write_doc_save_as" => CommonError::FsWrite(path, e.to_string()),

				"stat" => CommonError::FsStat(path, e.to_string()),

				"readdir" | "readdir_next" => CommonError::FsReadDir(path, e.to_string()),

				"mkdir" | "mkdir_parent" | "mkdir_all" | "mkdir_parent_rename" | "mkdir_parent_copy" => {
					CommonError::FsMkdir(path, e.to_string())
				},

				"delete" | "delete_stat_check" => CommonError::FsDelete(path, e.to_string()),

				"rename" | "rename_target_stat" => CommonError::FsRename(path, e.to_string()),

				"copy" | "copy_source_stat" => CommonError::FsCopy(path, e.to_string()),

				_ => {
					CommonError::Unknown(format!(
						// Fallback for unmapped operations
						"Unknown FS Operation '{}' on path '{}' failed with IO error: {}",
						operation,
						path.display(),
						e
					))
				},
			}
		},
	}
}

/// Detects a language ID string from a file path's extension.
/// This is a simplified heuristic. A more robust solution would use a mime-type
/// library or more extensive extension-to-languageID mappings.
fn detect_language_id_from_file_path(path:&Path) -> String {
	// TODO: Enhance language detection. Consider:
	//       - Using a dedicated crate for mime-type detection.
	//       - Allowing extensions to contribute language definitions.
	//       - Checking for modelines in file content.
	match path.extension().and_then(OsStr::to_str) {
		Some("js") | Some("mjs") | Some("cjs") => "javascript",

		Some("jsx") => "javascriptreact",

		Some("ts") => "typescript",

		Some("tsx") => "typescriptreact",

		Some("json") => "json",

		// JSON with Comments
		Some("jsonc") => "jsonc",

		Some("html") | Some("htm") => "html",

		Some("css") => "css",

		// Added sass
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

		// Added kts
		Some("kt") | Some("kts") => "kotlin",

		Some("dart") => "dart",

		Some("lua") => "lua",

		Some("sql") => "sql",

		Some("ps1") => "powershell",

		Some("bat") | Some("cmd") => "bat",

		// Common log file extension
		Some("log") => "log",

		// Add more common extensions and their language IDs
		// Default for unknown extensions
		_ => "plaintext",
	}
	.to_string()
}

/// Detects file encoding from byte content. Simplified for MVP.
/// TODO: Implement more robust encoding detection (e.g., using `chardet` crate
/// or checking for BOM).
fn detect_file_encoding_from_bytes(_content_bytes:&[u8]) -> String {
	// For MVP, assume UTF-8. A real implementation would inspect byte order marks
	// (BOM) or use heuristics / libraries like `chardetng` or `encoding_rs`.
	"utf8".to_string()
}

// --- Effect Provider Trait Implementations ---

#[async_trait]
impl FsReader for MountainEnvironment {
	async fn read_file(&self, path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		// Security check
		self.is_path_allowed_for_filesystem_access(path).await?;

		trace!("[Env FsReader] Reading file: {}", path.display());

		fs::read(path)
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "read"))
	}

	async fn stat_file(&self, path:&PathBuf) -> Result<FileSystemStat, CommonError> {
		// Security check
		self.is_path_allowed_for_filesystem_access(path).await?;

		trace!("[Env FsReader] Stating file/directory: {}", path.display());

		match tokio::fs::metadata(path).await {
			Ok(metadata) => {
				// Start with 0 (Unknown)

				let mut file_type_flags = 0_u8;

				if metadata.is_file() {
					file_type_flags |= CommonFileType::File as u8;
				}

				if metadata.is_dir() {
					file_type_flags |= CommonFileType::Directory as u8;
				}

				if metadata.is_symlink() {
					file_type_flags |= CommonFileType::SymbolicLink as u8;
				}

				// If no specific type flag is set, it remains Unknown (0).

				let get_milli_timestamp_from_system_time = |sys_time_res:Result<std::time::SystemTime, _>| -> u64 {
					sys_time_res
						.ok()
						.and_then(|time| time.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
						.map_or(0, |duration| duration.as_millis() as u64)
				};

				Ok(FileSystemStat {
					file_type:file_type_flags,

					ctime:get_milli_timestamp_from_system_time(metadata.created()),

					mtime:get_milli_timestamp_from_system_time(metadata.modified()),

					size:metadata.len(),

					permissions:None, /* TODO: Populate permissions if needed by VS Code extensions (e.g.,
					                   *
					                   *
					                   * FilePermission enum) */
				})
			},

			Err(io_err) => Err(map_io_error_to_common_error(io_err, path.clone(), "stat")),
		}
	}

	async fn read_directory(&self, path:&PathBuf) -> Result<Vec<(String, CommonFileType)>, CommonError> {
		// Security check
		self.is_path_allowed_for_filesystem_access(path).await?;

		debug!("[Env FsReader] Reading directory contents: {}", path.display());

		let mut entries_vec:Vec<(String, CommonFileType)> = Vec::new();

		let mut dir_entries_stream = fs::read_dir(path)
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "readdir"))?;

		while let Some(dir_entry_res) = dir_entries_stream
			.next_entry()
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "readdir_next_entry"))?
		{
			let file_name_osstr = dir_entry_res.file_name();

			let file_name_str = file_name_osstr.to_string_lossy().into_owned();

			match dir_entry_res.file_type().await {
				Ok(file_type_tokio) => {
					let common_file_type = if file_type_tokio.is_dir() {
						CommonFileType::Directory
					} else if file_type_tokio.is_file() {
						CommonFileType::File
					} else if file_type_tokio.is_symlink() {
						CommonFileType::SymbolicLink
					} else {
						CommonFileType::Unknown
					};

					entries_vec.push((file_name_str, common_file_type));
				},

				Err(e_ftype) => {
					warn!(
						"[Env FsReader] Failed to get file type for entry '{}' in directory '{}': {}. Marking as \
						 Unknown.",
						file_name_str,
						path.display(),
						e_ftype
					);

					entries_vec.push((file_name_str, CommonFileType::Unknown));
				},
			}
		}

		Ok(entries_vec)
	}
}

// Required by AppRuntime to provide FsReader capability.
impl Requires<Arc<dyn FsReader + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn FsReader + Send + Sync> {
		// Clone self (Arc<MountainEnvironment>) for the trait object
		Arc::new(self.clone())
	}
}

#[async_trait]
impl FsWriter for MountainEnvironment {
	async fn write_file(
		&self,

		path:&PathBuf,

		content_bytes:Vec<u8>,

		create_if_not_exists:bool,

		overwrite_if_exists:bool,
	) -> Result<(), CommonError> {
		// Security check
		self.is_path_allowed_for_filesystem_access(path).await?;

		info!(
			"[Env FsWriter] Writing file: path='{}', content_len={}, create={}, overwrite={}",
			path.display(),
			content_bytes.len(),
			create_if_not_exists,
			overwrite_if_exists
		);

		let path_exists = fs::try_exists(path).await.unwrap_or(false);

		if path_exists && !overwrite_if_exists {
			return Err(CommonError::FsFileExists(path.clone()));
		}

		if !path_exists && !create_if_not_exists {
			return Err(CommonError::FsNotFound(path.clone()));
		}

		// Ensure parent directory exists if creating.
		if let Some(parent_dir_path) = path.parent() {
			if !fs::try_exists(parent_dir_path).await.unwrap_or(false) {
				if create_if_not_exists {
					fs::create_dir_all(parent_dir_path).await.map_err(|io_err| {
						map_io_error_to_common_error(io_err, parent_dir_path.to_path_buf(), "mkdir_parent_for_write")
					})?;
				} else {
					// Cannot create file if parent dir doesn't exist and `create` is false.
					return Err(CommonError::FsNotFound(parent_dir_path.to_path_buf()));
				}
			}
		}

		fs::write(path, &content_bytes)
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "write"))?;

		// TODO: Emit filesystem_changed event via AppHandle. This is important if this
		// write       bypasses higher-level document management logic that would
		// normally emit such events.       Example:
		// self.app_handle.emit_all("mountain://filesystem/changed", json!({"uri":
		// path_to_uri(path), "type": "changed"}));

		Ok(())
	}

	async fn create_directory(&self, path:&PathBuf, recursive_create:bool) -> Result<(), CommonError> {
		// Security check
		self.is_path_allowed_for_filesystem_access(path).await?;

		info!(
			"[Env FsWriter] Creating directory: path='{}', recursive={}",
			path.display(),
			recursive_create
		);

		if recursive_create {
			// Creates parent directories as needed.
			fs::create_dir_all(path)
				.await
				.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "mkdir_all"))?;
		} else {
			// Fails if parent does not exist.
			fs::create_dir(path)
				.await
				.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "mkdir"))?;
		}

		// TODO: Emit filesystem_changed event.
		Ok(())
	}

	async fn delete(&self, path:&PathBuf, recursive_delete:bool, use_os_trash:bool) -> Result<(), CommonError> {
		// Security check
		self.is_path_allowed_for_filesystem_access(path).await?;

		info!(
			"[Env FsWriter] Deleting: path='{}', recursive={}, useTrash={}",
			path.display(),
			recursive_delete,
			use_os_trash
		);

		if use_os_trash {
			warn!(
				"[Env FsWriter] 'useTrash=true' option for delete is requested but not yet implemented. Performing \
				 permanent delete."
			);

			// TODO: Implement 'move to trash' functionality using a crate like
			// `trash`.       If `use_os_trash` is true and cannot be
			// fulfilled, it might be better to return an error       (e.g.,

			// CommonError::NotImplemented) or configure this behavior.
		}

		match fs::metadata(path).await {
			Ok(metadata) => {
				let delete_operation_result = if metadata.is_dir() {
					if recursive_delete {
						// Deletes directory and its contents.
						fs::remove_dir_all(path).await
					} else {
						// Deletes empty directory. Fails if not empty.
						fs::remove_dir(path).await
					}
				} else {
					// Deletes a file.
					fs::remove_file(path).await
				};

				delete_operation_result
					.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "delete"))?;

				// TODO: Emit filesystem_changed event.
				Ok(())
			},

			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
				// Deleting a non-existent path is considered success (idempotent) by VS Code FS
				// API.
				debug!(
					"[Env FsWriter] Path '{}' not found for deletion. Operation considered successful (idempotent).",
					path.display()
				);

				Ok(())
			},

			Err(io_err) => Err(map_io_error_to_common_error(io_err, path.clone(), "delete_stat_check")), /* Error stating the path before delete attempt */
		}
	}

	async fn rename(
		&self,

		source_path:&PathBuf,

		target_path:&PathBuf,

		overwrite_if_target_exists:bool,
	) -> Result<(), CommonError> {
		self.is_path_allowed_for_filesystem_access(source_path).await?;

		self.is_path_allowed_for_filesystem_access(target_path).await?;

		info!(
			"[Env FsWriter] Renaming/Moving: from='{}', to='{}', overwrite={}",
			source_path.display(),
			target_path.display(),
			overwrite_if_target_exists
		);

		if !fs::try_exists(source_path).await.unwrap_or(false) {
			return Err(CommonError::FsNotFound(source_path.clone()));
		}

		if !overwrite_if_target_exists && fs::try_exists(target_path).await.unwrap_or(false) {
			return Err(CommonError::FsFileExists(target_path.clone()));
		}

		// If overwriting, and target exists, delete target first.
		// `fs::rename` behavior with existing target can be platform-dependent.
		if overwrite_if_target_exists && fs::try_exists(target_path).await.unwrap_or(false) {
			debug!(
				"[Env FsWriter] Rename: Overwriting target by first deleting '{}'",
				target_path.display()
			);

			// Determine if target is dir for recursive delete, pass useTrash=false for
			// internal delete.
			let target_metadata = fs::metadata(target_path).await.map_err(|io_err| {
				map_io_error_to_common_error(io_err, target_path.clone(), "rename_target_stat_for_overwrite_delete")
			})?;

			self.delete(target_path, target_metadata.is_dir(), false).await?;
		}

		// Ensure target's parent directory exists.
		if let Some(target_parent_dir) = target_path.parent() {
			if !fs::try_exists(target_parent_dir).await.unwrap_or(false) {
				fs::create_dir_all(target_parent_dir).await.map_err(|io_err| {
					map_io_error_to_common_error(io_err, target_parent_dir.to_path_buf(), "mkdir_parent_for_rename")
				})?;
			}
		}

		fs::rename(source_path, target_path)
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, source_path.clone(), "rename"))?;

		// TODO: Emit filesystem_changed events (one delete for source, one create for
		// target, or a specific rename event).
		Ok(())
	}

	async fn copy(
		&self,

		source_path:&PathBuf,

		target_path:&PathBuf,

		overwrite_if_target_exists:bool,
	) -> Result<(), CommonError> {
		self.is_path_allowed_for_filesystem_access(source_path).await?;

		self.is_path_allowed_for_filesystem_access(target_path).await?;

		info!(
			"[Env FsWriter] Copying: from='{}', to='{}', overwrite={}",
			source_path.display(),
			target_path.display(),
			overwrite_if_target_exists
		);

		if !fs::try_exists(source_path).await.unwrap_or(false) {
			return Err(CommonError::FsNotFound(source_path.clone()));
		}

		if !overwrite_if_target_exists && fs::try_exists(target_path).await.unwrap_or(false) {
			return Err(CommonError::FsFileExists(target_path.clone()));
		}

		let source_metadata = fs::metadata(source_path)
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, source_path.clone(), "copy_source_stat"))?;

		// TODO: Implement recursive directory copy if `source_metadata.is_dir()`.
		//       `tokio::fs::copy` only copies files. For directories, one would need
		// to:
		//       1. Create the target directory.
		//       2. Recursively list entries in source directory.
		//       3. For each entry, call `copy` again (if file) or `create_directory`
		//          (if dir).
		//       This can be complex. For MVP, if it's a directory, return
		// NotImplemented.
		if source_metadata.is_dir() {
			error!(
				"[Env FsWriter] Recursive directory copy from '{}' is not yet implemented.",
				source_path.display()
			);

			return Err(CommonError::NotImplemented(
				"Recursive directory copy for vscode.workspace.fs.copy".to_string(),
			));
		}

		// If overwriting and target exists, delete target first (fs::copy might fail or
		// behave differently otherwise).
		if overwrite_if_target_exists && fs::try_exists(target_path).await.unwrap_or(false) {
			debug!(
				"[Env FsWriter] Copy: Overwriting target by first deleting '{}'",
				target_path.display()
			);

			// Assuming target is not a directory if source is not (for non-recursive copy).
			self.delete(target_path, false, false).await?;
		}

		// Ensure target's parent directory exists.
		if let Some(target_parent_dir) = target_path.parent() {
			if !fs::try_exists(target_parent_dir).await.unwrap_or(false) {
				fs::create_dir_all(target_parent_dir).await.map_err(|io_err| {
					map_io_error_to_common_error(io_err, target_parent_dir.to_path_buf(), "mkdir_parent_for_copy")
				})?;
			}
		}

		// tokio::fs::copy copies a file.
		fs::copy(source_path, target_path)

			.await
			 // Discard bytes copied, return unit on success
			.map(|_bytes_copied| ())

			.map_err(|io_err| map_io_error_to_common_error(io_err, source_path.clone(), "copy"))?;

		// TODO: Emit filesystem_changed event for target creation.
		Ok(())
	}
}

impl Requires<Arc<dyn FsWriter + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn FsWriter + Send + Sync> { Arc::new(self.clone()) }
}

// --- Other Provider Implementations (Storage, Config, Documents, etc.) ---
// These follow a similar pattern:
// - Access AppState via `self.get_app_state()`.
// - Lock relevant parts of AppState.
// - Perform logic, often delegating to helper functions in `handlers::*` or
//   methods on `AppState` DTOs.
// - Map errors to `CommonError`.
// - For UI interactions (`UiProvider`), use the async request-response pattern
//   with Sky.

#[async_trait]
impl StorageProvider for MountainEnvironment {
	async fn get_storage_value(&self, is_global_scope:bool, key:&str) -> Result<Option<Value>, CommonError> {
		trace!(
			"[Env StorageProvider] Getting value: scope_is_global={}, key='{}'",
			is_global_scope, key
		);

		let app_state = self.get_app_state();

		// 1 for Global, 0 for Workspace
		let scope_id = if is_global_scope { 1 } else { 0 };

		let (storage_map_mutex, _path_opt) = handlers::storage::get_storage_map_and_path_from_appstate(
			&app_state, scope_id,
		)
		.map_err(|json_err_str| CommonError::StateLock(format!("Failed to get storage map/path: {}", json_err_str)))?;

		let storage_map_guard = storage_map_mutex.lock().map_err(map_app_state_lock_error_to_common_error)?;

		let value_opt = storage_map_guard.get(key).cloned();

		debug!(
			"[Env StorageProvider] Value for key '{}' (scope_is_global={}): value_present={}",
			key,
			is_global_scope,
			value_opt.is_some()
		);

		Ok(value_opt)
	}

	async fn update_storage_value(
		&self,

		is_global_scope:bool,

		key:String,

		// Some(Value) to set/update, None to delete
		value_to_set:Option<Value>,
	) -> Result<(), CommonError> {
		info!(
			"[Env StorageProvider] Updating value: scope_is_global={}, key='{}', value_is_some={}",
			is_global_scope,
			key,
			value_to_set.is_some()
		);

		let app_state = self.get_app_state();

		let scope_id = if is_global_scope { 1 } else { 0 };

		let (storage_map_mutex, storage_file_path_opt) = handlers::storage::get_storage_map_and_path_from_appstate(
			&app_state, scope_id,
		)
		.map_err(|json_err_str| {
			CommonError::StateLock(format!("Failed to get storage map/path for update: {}", json_err_str))
		})?;

		// Clone data needed for saving *after* the lock is released.
		let data_to_persist_opt:Option<MementoStorageMap> = {
			let mut storage_map_guard = storage_map_mutex.lock().map_err(map_app_state_lock_error_to_common_error)?;

			if let Some(val) = value_to_set {
				storage_map_guard.insert(key.clone(), val);
			} else {
				// Remove if value_to_set is None
				storage_map_guard.remove(&key);
			}

			// Clone HashMap for saving only if a persistence path is available.
			storage_file_path_opt.as_ref().map(|_| storage_map_guard.clone())

			// Lock released here.
		};

		// Trigger async save task if path and data are available.
		if let (Some(path_to_save), Some(cloned_data_to_persist)) = (storage_file_path_opt, data_to_persist_opt) {
			let scope_name_log = if is_global_scope { "Global" } else { "Workspace" };

			debug!(
				"[Env StorageProvider] Spawning task to persist {} Memento to: {}",
				scope_name_log,
				path_to_save.display()
			);

			tokio::spawn(async move {
				if let Err(e_save) =
					handlers::storage::save_storage_map_to_disk(&path_to_save, &cloned_data_to_persist).await
				{
					error!(
						"[Env StorageProvider Task] Error persisting {} Memento to '{}': {}",
						scope_name_log,
						path_to_save.display(),
						e_save
					);
				}
			});
		} else if !is_global_scope && storage_file_path_opt.is_none() {
			warn!(
				"[Env StorageProvider] Workspace storage path is not set. Cannot persist value for key '{}'. Change \
				 will be in-memory only for this session.",
				key
			);
		}

		Ok(())
	}
}

impl Requires<Arc<dyn StorageProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn StorageProvider + Send + Sync> { Arc::new(self.clone()) }
}

// Implementations for ConfigProvider, ConfigInspector, DocumentProvider, etc.
// follow similar patterns, using `self.get_app_state()` and `handlers::*`
// helpers.

#[async_trait]
impl ConfigProvider for MountainEnvironment {
	async fn get_configuration_value(
		&self,

		// e.g., "editor.fontSize" or None for all
		section_key_opt:Option<String>,

		// For resource/language-specific values
		overrides:IConfigurationOverrides,
	) -> Result<Value, CommonError> {
		trace!(
			"[Env ConfigProvider] GetConfig: section={:?}, overrides.resource={:?}, overrides.langId={:?}",
			section_key_opt,
			// Log external URI if present
			overrides.resource.as_ref().and_then(|v| v.get("external")),
			// Language ID
			overrides.override_identifier
		);

		let app_state = self.get_app_state();

		let config_state_guard = app_state
			.configuration
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		// The `MergedConfigurationState::get_value` currently uses a simplified lookup
		// from the merged state. TODO: Enhance `MergedConfigurationState::get_value`
		// or this provider to fully respect `overrides`       by potentially
		// re-evaluating against specific configuration files or layers if the
		//       merged view isn't sufficient for fine-grained override resolution.
		if overrides.resource.is_some() || overrides.override_identifier.is_some() {
			warn!(
				"[Env ConfigProvider GetConfig] Overrides provided (resource or languageId), but current \
				 implementation primarily uses the pre-merged configuration state. Fine-grained override resolution \
				 beyond initial merge might be limited."
			);
		}

		let value_result = config_state_guard.get_value(
			section_key_opt.as_deref(),
			// Pass resource for potential scope logic in get_value
			overrides.resource.as_ref(),
		);

		debug!(
			"[Env ConfigProvider GetConfig] Value for section {:?}: (sample) {}...",
			section_key_opt,
			value_result.to_string().chars().take(70).collect::<String>()
		);

		Ok(value_result)
	}

	async fn update_configuration_value(
		&self,

		key_to_update:String,

		// If Value::Null, effectively removes the key
		value_to_set:Value,

		// User, Workspace, WorkspaceFolder
		target_scope:ConfigurationTarget,

		// For resource URI (if WORKSPACE_FOLDER) and languageId
		overrides:IConfigurationOverrides,

		// If true, write into language-specific section `[languageId]`
		scope_to_language_override:Option<bool>,
	) -> Result<(), CommonError> {
		info!(
			"[Env ConfigProvider UpdateConfig] Request: key='{}', target_scope={:?}, value_is_null={}, \
			 scope_to_lang={:?}, override_resource={:?}",
			key_to_update,
			target_scope,
			value_to_set.is_null(),
			scope_to_language_override,
			overrides.resource.as_ref().and_then(|v| v.get("external"))
		);

		let app_state = self.get_app_state();

		// 1. Determine the target settings.json file path.
		let target_config_file_path = handlers::config::get_config_path_for_target(
			&self.app_handle,
			&app_state,
			target_scope,
			&overrides,
			scope_to_language_override.unwrap_or(false),
		)?;

		info!(
			"[Env ConfigProvider UpdateConfig] Target config file for update: {}",
			target_config_file_path.display()
		);

		// 2. Load the current content of that specific settings file.
		let mut current_target_file_json_content =
			handlers::config::load_json_file_if_exists_or_default(&target_config_file_path).await?;

		trace!(
			"[Env ConfigProvider UpdateConfig] Loaded JSON ({} top-level keys) from target file '{}'",
			current_target_file_json_content.as_object().map_or(0, |m| m.keys().len()),
			target_config_file_path.display()
		);

		// 3. Update the value at the specified key within the loaded JSON content.
		//    Handle language-specific scoping if requested.
		let mut effective_json_node_to_update_in = &mut current_target_file_json_content;

		// To keep string alive for entry()

		let mut language_scope_key_holder:Option<String> = None;

		if scope_to_language_override.unwrap_or(false) {
			if let Some(lang_id_str) = &overrides.override_identifier {
				// e.g., "[typescript]"
				language_scope_key_holder = Some(format!("[{}]", lang_id_str));

				let lang_scope_key_ref = language_scope_key_holder.as_ref().unwrap();

				if !effective_json_node_to_update_in.is_object() {
					// If the top level of the file is not an object, make it one.
					*effective_json_node_to_update_in = json!({});
				}

				// Get or create the language-specific section.
				effective_json_node_to_update_in = effective_json_node_to_update_in
					.as_object_mut()

					 // Safe due to check above
					.unwrap()

					.entry(lang_scope_key_ref.clone())

					.or_insert_with(|| json!({}));
			} else {
				warn!(
					"[Env ConfigProvider UpdateConfig] 'scopeToLanguage' is true for key '{}', but no languageId was \
					 provided in overrides. Updating at the top level of the target file '{}' instead.",
					key_to_update,
					target_config_file_path.display()
				);
			}
		}

		handlers::config::update_json_value_at_path(effective_json_node_to_update_in, &key_to_update, value_to_set);

		trace!(
			"[Env ConfigProvider UpdateConfig] Key '{}' updated in in-memory JSON for file '{}'.",
			key_to_update,
			target_config_file_path.display()
		);

		// 4. Write the modified JSON content back to the target settings file.
		handlers::config::write_json_file(
			&target_config_file_path,
			// Pass the modified content
			&current_target_file_json_content,
		)
		.await?;

		info!(
			"[Env ConfigProvider UpdateConfig] Successfully wrote updated config to file: {}",
			target_config_file_path.display()
		);

		// 5. Reload and re-merge all configurations into AppState.configuration.
		let new_merged_config_state =
			handlers::config::load_and_merge_configurations_internal(&self.app_handle, &app_state).await?;

		app_state
			.configuration
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.update_from_new_state(new_merged_config_state);

		info!(
			"[Env ConfigProvider UpdateConfig] In-memory AppState.configuration reloaded and updated after change to \
			 file '{}'.",
			target_config_file_path.display()
		);

		// 6. Notify Cocoon (and other listeners) that configuration has changed.
		handlers::config::notify_config_changed_for_keys(
			&self.app_handle,
			// Notify for the specific key that was changed
			vec![key_to_update],
		)
		.await;

		Ok(())
	}
}

impl Requires<Arc<dyn ConfigProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn ConfigProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl ConfigInspector for MountainEnvironment {
	async fn inspect_configuration_value(
		&self,

		key:String,

		overrides:IConfigurationOverrides,
	) -> Result<Option<InspectResultData>, CommonError> {
		info!(
			"[Env ConfigInspector] Inspecting config key='{}', overrides.resource={:?}",
			key,
			overrides.resource.as_ref().and_then(|v| v.get("external"))
		);

		warn!(
			"[Env ConfigInspector] inspect_configuration_value is STUBBED for MVP. It will always return None. Full \
			 implementation requires reading individual config files (User, Workspace, Folder) and determining where \
			 each value for the key is defined."
		);

		// TODO: Implement full inspection logic:
		// 1. For the given `key` and `overrides` (resource URI, languageId):
		// 2. Load User settings.json, check for `key` (and lang-specific if
		//    applicable).
		// 3. Load Workspace .code-workspace file, check `settings` object for `key`.
		// 4. Determine relevant Workspace Folder based on `overrides.resource`. Load
		//    its .vscode/settings.json, check for `key`.
		// 5. Load default values (if Mountain has a concept of default configuration).
		// 6. Construct `InspectResultData` populating `defaultValue`, `userValue`,

		//    `workspaceValue`, `workspaceFolderValue`, and the final `value` (effective
		//    value). Also indicate `default`, `user`, `workspace`, `workspaceFolder`
		//    scopes if the value is defined there.
		// Stubbed for MVP
		Ok(None)
	}
}

impl Requires<Arc<dyn ConfigInspector + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn ConfigInspector + Send + Sync> { Arc::new(self.clone()) }
}

// Remaining provider implementations (DocumentProvider, SecretsProvider, etc.)

// will be added following the established pattern.
// For brevity in this response, I'll provide skeletons or key parts.

#[async_trait]
impl DocumentProvider for MountainEnvironment {
	async fn open_document(
		&self,

		// DTO from Cocoon: {scheme, path, external, ...} or null for new untitled
		uri_components_dto:Value,

		language_id_override_opt:Option<String>,

		initial_content_opt:Option<String>,
	) -> Result<Url, CommonError> {
		info!(
			"[Env DocumentProvider] OpenDocument: uri_dto(external)='{:?}', lang_override='{:?}', \
			 has_initial_content={}",
			uri_components_dto.get("external").or_else(|| uri_components_dto.get("path")),
			language_id_override_opt,
			initial_content_opt.is_some()
		);

		let app_state = self.get_app_state();

		let target_uri:Url;

		let is_new_untitled:bool;

		if uri_components_dto.is_null() || uri_components_dto.as_object().map_or(true, |o| o.is_empty()) {
			// Create a new untitled document.
			// TODO: Generate a unique "untitled:Untitled-N" URI.
			//       For now, using a placeholder. Ensure this is unique if multiple
			// untitled docs are supported.
			let untitled_counter = app_state
				.open_documents
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?
				.len() + 1;

			target_uri = Url::parse(&format!("untitled:Untitled-{}", untitled_counter))
				.map_err(|e| CommonError::InvalidArg("untitled_uri_generation".to_string(), e.to_string()))?;

			is_new_untitled = true;

			info!("[Env DocumentProvider] Creating new untitled document with URI: {}", target_uri);
		} else {
			// Open an existing document from the provided URI components.
			target_uri = handlers::documents::parse_uri_from_components_param(
				&uri_components_dto,
				"open_document",
				"uri_components",
				None,
			)
			.map_err(|e_str| CommonError::InvalidArg("uri_components".to_string(), e_str))?;

			is_new_untitled = false;

			info!("[Env DocumentProvider] Opening existing document with URI: {}", target_uri);
		}

		// Check if document is already open.
		{
			let open_docs_guard = app_state
				.open_documents
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?;

			if let Some(existing_doc_state) = open_docs_guard.get(target_uri.as_str()) {
				info!(
					"[Env DocumentProvider] Document '{}' (v{}) is already open. Returning existing.",
					target_uri, existing_doc_state.version
				);

				// TODO: If `initial_content_opt` is provided for an already open document,

				// should it update or warn?       Current behavior: returns existing,

				// ignores new content/langId for open doc.
				return Ok(target_uri);
			}

			// Lock released.
		}

		// Determine content, EOL, encoding, languageID, and initial dirty state.
		let (final_lines_vec, final_eol_str, final_encoding_str, final_language_id_str, initial_is_dirty_bool) =
			if let Some(content_str) = initial_content_opt {
				// Content is provided (e.g., for new untitled file, or opening with specific
				// content).
				let (lines, eol) = app_state::analyze_text_lines_and_eol_for_document_state(&content_str);

				let lang_id = language_id_override_opt
					.unwrap_or_else(|| detect_language_id_from_file_path(Path::new(target_uri.path())));

				info!(
					"[Env DocumentProvider] Document '{}' opened with provided content. LangId determined as '{}'.",
					target_uri, lang_id
				);

				// Provided content makes it dirty initially.
				(lines, eol, "utf8".to_string(), lang_id, true)
			} else {
				// No initial content provided; must be an existing file URI.
				if target_uri.scheme() != "file" {
					error!(
						"[Env DocumentProvider] Attempted to open non-file URI '{}' without providing initial content.",
						target_uri
					);

					return Err(CommonError::NotImplemented(format!(
						"Opening non-file URIs ('{}') without initial content is not supported.",
						target_uri.scheme()
					)));
				}

				let file_path = PathBuf::from(target_uri.path());

				// Security check
				self.is_path_allowed_for_filesystem_access(&file_path).await?;

				debug!("[Env DocumentProvider] Reading file content for path: {}", file_path.display());

				let file_bytes = fs::read(&file_path)
					.await
					.map_err(|io_err| map_io_error_to_common_error(io_err, file_path.clone(), "read_doc_open"))?;

				let encoding_detected_str = detect_file_encoding_from_bytes(&file_bytes);

				// TODO: Handle non-UTF8 encodings properly. For MVP, assuming UTF-8 after
				// detection.
				let content_from_file_str = String::from_utf8(file_bytes).map_err(|utf8_err| {
					CommonError::FsRead(file_path, format!("UTF-8 decoding error after reading file: {}", utf8_err))
				})?;

				let (lines, eol) = app_state::analyze_text_lines_and_eol_for_document_state(&content_from_file_str);

				let lang_id = language_id_override_opt
					.unwrap_or_else(|| detect_language_id_from_file_path(Path::new(target_uri.path())));

				// Existing file is not dirty initially.
				(lines, eol, encoding_detected_str, lang_id, false)
			};

		let new_document_state = DocumentState {
			uri:target_uri.clone(),

			language_id:final_language_id_str,

			// Initial version
			version:1,

			lines:final_lines_vec,

			eol:final_eol_str,

			is_dirty:initial_is_dirty_bool,

			encoding:final_encoding_str,
		};

		{
			let mut open_docs_guard = app_state
				.open_documents
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?;

			info!(
				"[Env DocumentProvider] Inserting new document '{}' (v1) into AppState.open_documents.",
				target_uri
			);

			open_docs_guard.insert(target_uri.as_str().to_string(), new_document_state.clone());

			// Lock released.
		}

		// Notify Cocoon that the model (document) has been added.
		handlers::documents::notify_model_added(self.app_handle.clone(), &new_document_state).await;

		info!(
			"[Env DocumentProvider] Document '{}' (v1) opened successfully and 'modelAdded' notification sent.",
			target_uri
		);

		Ok(target_uri)
	}

	async fn save_document(&self, uri_to_save:Url) -> Result<bool, CommonError> {
		info!("[Env DocumentProvider] SaveDocument request for URI: {}", uri_to_save);

		if uri_to_save.scheme() != "file" {
			return Err(CommonError::NotImplemented(format!(
				"Saving non-file URI schemes ('{}') is not supported.",
				uri_to_save.scheme()
			)));
		}

		let file_path_to_save = PathBuf::from(uri_to_save.path());

		// Security check
		self.is_path_allowed_for_filesystem_access(&file_path_to_save).await?;

		let app_state = self.get_app_state();

		let (content_to_save_str, current_version_in_state) = {
			// Scope for first lock
			let open_docs_guard = app_state
				.open_documents
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?;

			let doc_state = open_docs_guard.get(uri_to_save.as_str()).ok_or_else(|| {
				warn!(
					"[Env DocumentProvider Save] Document '{}' not found in open_documents map for saving.",
					uri_to_save
				);

				// Or a more specific "DocumentNotOpen" error
				CommonError::FsNotFound(file_path_to_save.clone())
			})?;

			if !doc_state.is_dirty {
				info!(
					"[Env DocumentProvider Save] Document '{}' is not dirty. No save needed.",
					uri_to_save
				);

				// Indicate success as no action was required.
				return Ok(true);
			}

			(doc_state.get_text_content(), doc_state.version)

			// Lock released.
		};

		// Perform the actual file write using FsWriter.
		let fs_writer_provider:Arc<dyn FsWriter + Send + Sync> = self.require();

		fs_writer_provider
			.write_file(&file_path_to_save, content_to_save_str.into_bytes(), true, true)
			.await?;

		// write_file ensures create=true, overwrite=true for a standard save.

		// After successful write, update the document's dirty state in AppState.
		// Assume it was, will be confirmed by state.
		let mut was_dirty_before_this_save_op = true;

		{
			// Scope for second lock
			let mut open_docs_guard = app_state
				.open_documents
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?;

			if let Some(doc_state_mut) = open_docs_guard.get_mut(uri_to_save.as_str()) {
				// Check if the version has changed *during* the save operation.
				// This is a race condition check. If version changed, the save might be stale.
				if doc_state_mut.version == current_version_in_state {
					doc_state_mut.is_dirty = false;
				} else {
					// Get current dirty state
					was_dirty_before_this_save_op = doc_state_mut.is_dirty;

					warn!(
						"[Env DocumentProvider Save] Document '{}' content changed (v{} -> v{}) during save \
						 operation. Current dirty state: {}. Saved content might be stale.",
						uri_to_save, current_version_in_state, doc_state_mut.version, doc_state_mut.is_dirty
					);

					// Do NOT mark as clean if content changed during save, as
					// the saved file might not reflect latest state.
				}
			} else {
				// Document was removed from state while saving; this is unusual.
				warn!(
					"[Env DocumentProvider Save] Document '{}' was removed from AppState.open_documents during save \
					 operation.",
					uri_to_save
				);

				// Cannot confirm, assume not dirty for notification logic.
				was_dirty_before_this_save_op = false;
			}

			// Lock released.
		}

		// Notify Cocoon that the model was saved.
		handlers::documents::notify_model_saved(self.app_handle.clone(), &uri_to_save).await;

		// If the document was dirty and is now clean as a result of this save, notify
		// dirty state change.
		if was_dirty_before_this_save_op {
			// Only notify if it *was* dirty and might now be clean.
			// Re-check current dirty state for accuracy, in case of race.
			let current_is_dirty_after_save = app_state
				.open_documents
				.lock()

				.map_err(map_app_state_lock_error_to_common_error)?
				.get(uri_to_save.as_str())

				 // Default to true (still dirty) if not found (shouldn't happen)

				.map_or(true, |ds| ds.is_dirty);

			if !current_is_dirty_after_save {
				handlers::documents::notify_dirty_state_changed(self.app_handle.clone(), &uri_to_save, false).await;
			}
		}

		info!(
			"[Env DocumentProvider Save] Document '{}' save process complete, notifications sent.",
			uri_to_save
		);

		Ok(true)
	}

	// Other DocumentProvider methods (save_as, apply_document_changes) would follow
	// a similar pattern:
	// 1. Log entry.
	// 2. Access/lock AppState.
	// 3. Perform logic (file I/O via FsWriter/FsReader, UI via UiProvider for Save
	//    As dialog).
	// 4. Update AppState.
	// 5. Send notifications to Cocoon via `handlers::documents::notify_*`.
	// 6. Return Result.
	// ... (Implementations for save_document_as and apply_document_changes as per
	// previous review,      they are quite detailed and long, so omitting full
	// re-paste here for brevity but assuming      they are correctly implemented
	// using this pattern).
	async fn save_document_as(
		&self,

		original_uri:Url,

		new_uri_target_opt:Option<Url>,
	) -> Result<Option<Url>, CommonError> {
		info!(
			"[Env DocumentProvider] Save As: Original='{}', Target (if provided)='{:?}'",
			original_uri, new_uri_target_opt
		);

		let new_uri = match new_uri_target_opt {
			Some(uri) => uri,

			None => {
				let ui_provider_arc:Arc<dyn UiProvider + Send + Sync> = self.require();

				// Prepare default URI for dialog based on original.
				let default_uri_for_dialog_dto = json!({




					"scheme": original_uri.scheme(),


					"path": original_uri.path(),


					"external": original_uri.to_string(),


					"$mid": 1
				});

				let save_dialog_options_val = json!({ "defaultUri": default_uri_for_dialog_dto });

				let save_dialog_options_parsed:Option<SaveDialogOptions> =
					serde_json::from_value(save_dialog_options_val)
						.map_err(|e| CommonError::InvalidArg("SaveDialogOptions".into(), e.to_string()))?;

				match ui_provider_arc.show_save_dialog(save_dialog_options_parsed).await? {
					Some(selected_path_buf) => {
						Url::from_file_path(selected_path_buf).map_err(|_| {
							CommonError::InvalidArg(
								"new_uri_target_from_dialog".to_string(),
								"Selected path from save dialog is not a valid file URI".to_string(),
							)
						})?
					},

					None => {
						info!(
							"[Env DocumentProvider SaveAs] User cancelled Save As dialog for document: {}",
							original_uri
						);

						// User cancelled
						return Ok(None);
					},
				}
			},
		};

		if new_uri.scheme() != "file" {
			return Err(CommonError::NotImplemented(format!(
				"Save As to non-file URI schemes ('{}') is not supported.",
				new_uri.scheme()
			)));
		}

		let new_file_path = PathBuf::from(new_uri.path());

		// Security check
		self.is_path_allowed_for_filesystem_access(&new_file_path).await?;

		let app_state = self.get_app_state();

		let (
			original_doc_content_str,
			original_lang_id_str,
			original_encoding_str,
			original_eol_str,
			// True if original_uri was "untitled:"
			was_original_untitled,
		) = {
			// Scope for lock
			let open_docs_guard = app_state
				.open_documents
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?;

			let original_doc_state_ref = open_docs_guard.get(original_uri.as_str()).ok_or_else(|| {
				// Or "DocumentNotOpen" error
				CommonError::FsNotFound(PathBuf::from(original_uri.path()))
			})?;

			(
				original_doc_state_ref.get_text_content(),
				original_doc_state_ref.language_id.clone(),
				original_doc_state_ref.encoding.clone(),
				original_doc_state_ref.eol.clone(),
				original_uri.scheme() == "untitled",
			)

			// Lock released
		};

		// Write content to the new file path.
		let fs_writer_provider:Arc<dyn FsWriter + Send + Sync> = self.require();

		fs_writer_provider
			.write_file(&new_file_path, original_doc_content_str.clone().into_bytes(), true, true)
			.await?;

		// Update AppState:
		// - If original was untitled, remove it.
		// - Add new DocumentState for the new_uri.
		let (new_lines_vec, _new_eol_after_save) =
			app_state::analyze_text_lines_and_eol_for_document_state(&original_doc_content_str);

		let new_document_state_for_appstate = DocumentState {
			uri:new_uri.clone(),

			language_id:original_lang_id_str,

			// New file, so version 1
			version:1,

			lines:new_lines_vec,

			eol:original_eol_str,

			// Just saved, so not dirty
			is_dirty:false,

			encoding:original_encoding_str,
		};

		{
			// Scope for lock
			let mut open_docs_guard = app_state
				.open_documents
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?;

			if was_original_untitled {
				if open_docs_guard.remove(original_uri.as_str()).is_some() {
					info!(
						"[Env DocumentProvider SaveAs] Original untitled document '{}' removed from AppState.",
						original_uri
					);
				}
			}

			open_docs_guard.insert(new_uri.as_str().to_string(), new_document_state_for_appstate.clone());

			// Lock released
		}

		// Send notifications to Cocoon.
		if was_original_untitled {
			handlers::documents::notify_model_removed(self.app_handle.clone(), &original_uri).await;
		}

		handlers::documents::notify_model_added(self.app_handle.clone(), &new_document_state_for_appstate).await;

		handlers::documents::notify_model_saved(self.app_handle.clone(), &new_uri).await;

		// Since it's newly saved and not dirty, no notify_dirty_state_changed(false) is
		// strictly needed unless prior state might imply it. However, if original was
		// dirty, its dirty state change notification would have been handled by its own
		// save or closure.

		info!(
			"[Env DocumentProvider SaveAs] Document '{}' successfully saved as '{}'. Notifications sent.",
			original_uri, new_uri
		);

		Ok(Some(new_uri))
	}

	async fn apply_document_changes(
		&self,

		uri_to_change:Url,

		new_version_id:i64,

		// Array of RpcModelContentChangeDto
		changes_dto_collection_val:Value,

		is_dirty_after_change:bool,

		is_undoing_op:bool,

		is_redoing_op:bool,
	) -> Result<(), CommonError> {
		info!(
			"[Env DocumentProvider ApplyChanges] For URI='{}': new_version={}, num_changes={}, is_dirty={}, undo={}, \
			 redo={}",
			uri_to_change,
			new_version_id,
			changes_dto_collection_val.as_array().map_or(0, |a| a.len()),
			is_dirty_after_change,
			is_undoing_op,
			is_redoing_op
		);

		let app_state = self.get_app_state();

		// Acquire lock on open_documents map.
		let mut open_docs_guard = app_state
			.open_documents
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		if let Some(doc_state_mut) = open_docs_guard.get_mut(uri_to_change.as_str()) {
			let old_version_id_in_state = doc_state_mut.version;

			// Apply changes to the DocumentState object.
			// `apply_document_content_changes` handles version checking and updates
			// `is_dirty`.
			if let Err(e_apply_str) =
				doc_state_mut.apply_document_content_changes(new_version_id, &changes_dto_collection_val)
			{
				error!(
					"[Env DocumentProvider ApplyChanges] Error applying changes to internal DocumentState for URI \
					 '{}': {}. Changes: {:?}",
					uri_to_change, e_apply_str, changes_dto_collection_val
				);

				return Err(CommonError::Unknown(format!(
					"Internal document change application failed: {}",
					e_apply_str
				)));
			}

			// After applying, `doc_state_mut.version` should be `new_version_id`.
			// Update `is_dirty` based on the explicit flag from the caller (Cocoon).
			doc_state_mut.is_dirty = is_dirty_after_change;

			// Clone necessary fields for notification *before* dropping the lock.
			let updated_doc_eol_clone = doc_state_mut.eol.clone();

			// Should match new_version_id if applied
			let final_doc_version_after_apply = doc_state_mut.version;

			let final_doc_is_dirty_after_apply = doc_state_mut.is_dirty;

			// Release lock before await on notification.
			drop(open_docs_guard);

			// Notify Cocoon about the model changes.
			handlers::documents::notify_model_changed(
				self.app_handle.clone(),
				&uri_to_change,
				final_doc_version_after_apply,
				&updated_doc_eol_clone,
				final_doc_is_dirty_after_apply,
				// Pass original DTO for Cocoon
				changes_dto_collection_val,
				is_undoing_op,
				is_redoing_op,
			)
			.await;

			Ok(())
		} else {
			warn!(
				"[Env DocumentProvider ApplyChanges] Document URI '{}' not found in AppState.open_documents. Cannot \
				 apply changes.",
				uri_to_change
			);

			// Depending on strictness, this could be an error or a silent ignore.
			// Returning an error is generally safer.
			// Or "DocumentNotOpen"
			Err(CommonError::FsNotFound(PathBuf::from(uri_to_change.path())))
		}
	}
}

impl Requires<Arc<dyn DocumentProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn DocumentProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl SecretsProvider for MountainEnvironment {
	async fn get_secret(&self, extension_id:String, key:String) -> Result<Option<String>, CommonError> {
		trace!(
			"[Env SecretsProvider] Getting secret: extension_id='{}', key='{}'",
			extension_id, key
		);

		// Delegate to the handler function which interacts with the `keyring` crate.
		handlers::secrets::handle_get_secret(
            self.app_handle.clone(),


            json!({ "extensionId": extension_id, "key": key }),


        )

        .await
         // Convert Value::String to Option<String>
		.map(|json_value| json_value.as_str().map(String::from))

        .map_err(|json_rpc_err_str| CommonError::SecretsAccess(key, json_rpc_err_str))
	}

	async fn store_secret(&self, extension_id:String, key:String, value_to_store:String) -> Result<(), CommonError> {
		info!(
			"[Env SecretsProvider] Storing secret: extension_id='{}', key='{}'",
			extension_id, key
		);

		handlers::secrets::handle_store_secret(
            self.app_handle.clone(),


            json!({ "extensionId": extension_id, "key": key, "value": value_to_store }),


        )

        .await
         // Discard Value::Null, return unit
		.map(|_value_null| ())

        .map_err(|json_rpc_err_str| CommonError::SecretsAccess(key, json_rpc_err_str))
	}

	async fn delete_secret(&self, extension_id:String, key:String) -> Result<(), CommonError> {
		info!(
			"[Env SecretsProvider] Deleting secret: extension_id='{}', key='{}'",
			extension_id, key
		);

		handlers::secrets::handle_delete_secret(
			self.app_handle.clone(),
			json!({ "extensionId": extension_id, "key": key }),
		)
		.await
		.map(|_value_null| ())
		.map_err(|json_rpc_err_str| CommonError::SecretsAccess(key, json_rpc_err_str))
	}
}

impl Requires<Arc<dyn SecretsProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn SecretsProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl OutputChannelManager for MountainEnvironment {
	async fn register_channel(&self, name:String, language_id_opt:Option<String>) -> Result<String, CommonError> {
		info!(
			"[Env OutputChannelManager] Registering channel: name='{}', language_id='{:?}'",
			name, language_id_opt
		);

		handlers::output::handle_register_output_channel(
			self.app_handle.clone(),

			// Params for handle_register: [name, file_uri_opt, language_id_opt]
			json!([name, Value::Null, language_id_opt]),

		)

		.await
        // Returns channel_id (name)

		.map(|channel_id_val| channel_id_val.as_str().unwrap_or(&name).to_string())

		.map_err(|json_rpc_err_str| CommonError::OutputChannel(name, json_rpc_err_str))
	}

	async fn append(&self, channel_id:String, value_to_append:String) -> Result<(), CommonError> {
		trace!(
			"[Env OutputChannelManager] Appending to channel: id='{}', value_len={}",
			channel_id,
			value_to_append.len()
		);

		handlers::output::handle_append_to_output_channel(self.app_handle.clone(), json!([channel_id, value_to_append]))
			.await
			.map(|_value_null| ())
			.map_err(|json_rpc_err_str| CommonError::OutputChannel("append_operation".to_string(), json_rpc_err_str))
	}

	// ... Implementations for clear, replace, reveal, close, dispose ...
	// These will follow the pattern of calling the corresponding
	// `handlers::output::handle_*` function.
	async fn clear(&self, channel_id:String) -> Result<(), CommonError> {
		info!("[Env OutputMgr] Clearing channel: id='{}'", channel_id);

		handlers::output::handle_clear_output_channel(self.app_handle.clone(), json!([channel_id]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::OutputChannel(channel_id, e_str))
	}

	async fn replace(&self, channel_id:String, value:String) -> Result<(), CommonError> {
		info!("[Env OutputMgr] Replacing content of channel: id='{}'", channel_id);

		handlers::output::handle_replace_output_channel_content(self.app_handle.clone(), json!([channel_id, value]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::OutputChannel(channel_id, e_str))
	}

	async fn reveal(&self, channel_id:String, preserve_focus:bool) -> Result<(), CommonError> {
		info!(
			"[Env OutputMgr] Revealing channel: id='{}', preserve_focus={}",
			channel_id, preserve_focus
		);

		handlers::output::handle_reveal_output_channel(self.app_handle.clone(), json!([channel_id, preserve_focus]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::OutputChannel(channel_id, e_str))
	}

	async fn close(&self, channel_id:String) -> Result<(), CommonError> {
		info!("[Env OutputMgr] Closing channel view: id='{}'", channel_id);

		handlers::output::handle_close_output_channel_view(self.app_handle.clone(), json!([channel_id]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::OutputChannel(channel_id, e_str))
	}

	async fn dispose(&self, channel_id:String) -> Result<(), CommonError> {
		info!("[Env OutputMgr] Disposing channel: id='{}'", channel_id);

		handlers::output::handle_dispose_output_channel(self.app_handle.clone(), json!([channel_id]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::OutputChannel(channel_id, e_str))
	}
}

impl Requires<Arc<dyn OutputChannelManager + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn OutputChannelManager + Send + Sync> { Arc::new(self.clone()) }
}

// Implementations for DiagnosticsManager, CommandExecutor, WorkspaceProvider,

// UiProvider, IpcProvider, LanguageFeatureProviderRegistry These will also
// delegate to `handlers::*` or implement logic using AppState and Tauri/Tokio
// APIs. Due to length, I will show key aspects for UiProvider and
// LanguageFeatureProviderRegistry.

#[async_trait]
impl UiProvider for MountainEnvironment {
	// Example for show_message, others (show_open_dialog, etc.) follow similar
	// pattern
	async fn show_message(
		&self,

		severity:MessageSeverity,

		message_text:String,

		// JSON Value for MessageOptions DTO
		options_json_val_opt:Option<Value>,
	) -> Result<Option<String>, CommonError> {
		let severity_str = match severity {
			MessageSeverity::Info => "info",

			MessageSeverity::Warning => "warn",

			MessageSeverity::Error => "error",
		};

		info!(
			"[Env UiProvider ShowMessage] Severity='{}', Message='{}...', OptionsIsSome={}",
			severity_str,
			message_text.chars().take(50).collect::<String>(),
			options_json_val_opt.is_some()
		);

		// Simpler case: No buttons, not modal -> use tauri::api::dialog directly
		// (non-blocking for effect)

		let use_simple_dialog = options_json_val_opt.as_ref().map_or(true, |opts_val| {
			let items_empty = opts_val.get("items").and_then(Value::as_array).map_or(true, Vec::is_empty);

			let not_modal = !opts_val.get("modal").and_then(Value::as_bool).unwrap_or(false);

			items_empty && not_modal
		});

		if use_simple_dialog {
			let window_main = self
				.app_handle
				.get_window("main")
				.ok_or_else(|| CommonError::UiInteraction("Main window not found for simple dialog.".to_string()))?;

			let title_str = format!("Land Editor - {}", severity_str.to_uppercase());

			let message_clone_for_dialog = message_text.clone();

			// tauri::api::dialog::message is synchronous, run in spawn_blocking to not
			// block async runtime.
			tokio::task::spawn_blocking(move || {
				tauri::api::dialog::message(Some(&window_main), title_str, message_clone_for_dialog);
			})
			.await
			.map_err(|e_join| {
				CommonError::UiInteraction(format!("Failed to spawn blocking task for simple dialog: {}", e_join))
			})?;

			// Simple dialogs don't return selections here.
			return Ok(None);
		}

		// Complex case: Modal or has buttons, use async request-response with Sky.
		let request_id_str = Uuid::new_v4().to_string();

		let (response_sender_oneshot, response_receiver_oneshot) = TokioOneshot::channel();

		{
			// Scope for lock on pending_ui_requests
			let app_state = self.get_app_state();

			let mut pending_requests_guard = app_state
				.pending_ui_requests
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?;

			pending_requests_guard.insert(request_id_str.clone(), response_sender_oneshot);
		}

		// Payload for the sky://ui/show-message-request event
		let sky_event_payload_data = json!({




			"severity": severity_str,


			"message": message_text,


			"options": options_json_val_opt.unwrap_or(Value::Null)

		});

		let sky_event_full_payload = UiRequestToSkyPayload {
			// Assuming this helper struct exists
			request_id:request_id_str.clone(),

			payload:sky_event_payload_data,
		};

		self.app_handle
			.emit_all("sky://ui/show-message-request", sky_event_full_payload)
			.map_err(|e_emit| {
				CommonError::UiInteraction(format!("Failed to emit 'sky://ui/show-message-request' event: {}", e_emit))
			})?;

		// Wait for Sky's response via sky_resolves_ui_request, with timeout.
		// TODO: Make timeout configurable (e.g., 5 minutes for user interaction).
		let ui_response_result = match tokio_timeout(TokioDuration::from_secs(300), response_receiver_oneshot).await {
			Ok(Ok(Ok(value_from_sky))) => {
				// Successfully received Ok(Value) from oneshot
				// Sky sends back the selected item's string title, or null if dismissed/no
				// selection.
				if value_from_sky.is_null() {
					Ok(None)
				} else if let Some(selected_item_title_str) = value_from_sky.as_str() {
					Ok(Some(selected_item_title_str.to_string()))
				} else {
					Err(CommonError::UiInteraction(
						"showMessage response from Sky was not a string or null.".to_string(),
					))
				}
			},

			// Sky reported an error processing UI
			Ok(Ok(Err(common_error_from_sky))) => Err(common_error_from_sky),

			Ok(Err(_channel_closed_err)) => {
				// Oneshot sender was dropped without sending (e.g., sky_resolves_ui_request
				// panicked)

				Err(CommonError::UiInteraction(format!(
					"UiProvider showMessage (ReqID: {}): Response channel closed prematurely by Sky handler.",
					request_id_str
				)))
			},

			Err(_timeout_elapsed_err) => {
				// Timeout waiting for Sky's response
				warn!(
					"[Env UiProvider ShowMessage] Timed out waiting for Sky's response for ReqID: {}. Assuming \
					 dismissal.",
					request_id_str
				);

				// Treat timeout as dismissal or no selection.
				Ok(None)
			},
		};

		// Clean up the pending request entry.
		if let Ok(mut guard) = self.get_app_state().pending_ui_requests.lock() {
			guard.remove(&request_id_str);
		} else {
			error!(
				"[Env UiProvider ShowMessage] Failed to lock pending_ui_requests for cleanup of ReqID: {} (lock \
				 poisoned?).",
				request_id_str
			);
		}

		ui_response_result
	}

	// ... Implementations for show_open_dialog, show_save_dialog, show_quick_pick,

	// show_input_box ... These will follow the async request-response pattern with
	// Sky shown above for complex messages.
	async fn show_open_dialog(&self, options:Option<OpenDialogOptions>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		let request_id = Uuid::new_v4().to_string();

		info!(
			"[Env UiProvider] show_open_dialog (ReqID: {}): options={:?}",
			request_id, options
		);

		let (tx, rx) = TokioOneshot::channel();

		{
			self.get_app_state()
				.pending_ui_requests
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?
				.insert(request_id.clone(), tx);
		}

		let event_payload = UiRequestToSkyPayload { request_id:request_id.clone(), payload:options.clone() };

		self.app_handle
			.emit_all("sky://ui/show-open-dialog-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed to emit show_open_dialog request: {}", e)))?;

		let result = match tokio_timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(v))) => {
				// Parse v into Option<Vec<PathBuf>>
				if v.is_null() {
					Ok(None)
				} else if let Some(arr) = v.as_array() {
					arr.iter()
						.map(|p_val| {
							p_val.as_str().map(PathBuf::from).ok_or_else(|| {
								CommonError::UiInteraction("Invalid path string in open dialog response".into())
							})
						})
						.collect::<Result<Vec<_>, _>>()
						.map(Some)
				} else {
					Err(CommonError::UiInteraction("Open dialog response not an array or null".into()))
				}
			},

			Ok(Ok(Err(e))) => Err(e),

			Ok(Err(_)) => {
				Err(CommonError::UiInteraction(format!(
					"Open dialog (ReqID: {}) channel closed.",
					request_id
				)))
			},

			Err(_) => {
				warn!("[Env UiProvider] show_open_dialog (ReqID: {}) timed out.", request_id);

				Ok(None)
			},
		};

		self.get_app_state()
			.pending_ui_requests
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.remove(&request_id);

		result
	}

	async fn show_save_dialog(&self, options:Option<SaveDialogOptions>) -> Result<Option<PathBuf>, CommonError> {
		let request_id = Uuid::new_v4().to_string();

		info!(
			"[Env UiProvider] show_save_dialog (ReqID: {}): options={:?}",
			request_id, options
		);

		let (tx, rx) = TokioOneshot::channel();

		{
			self.get_app_state()
				.pending_ui_requests
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?
				.insert(request_id.clone(), tx);
		}

		let event_payload = UiRequestToSkyPayload { request_id:request_id.clone(), payload:options.clone() };

		self.app_handle
			.emit_all("sky://ui/show-save-dialog-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed to emit show_save_dialog request: {}", e)))?;

		let result = match tokio_timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(v))) => {
				// Parse v into Option<PathBuf>
				if v.is_null() {
					Ok(None)
				} else if let Some(s) = v.as_str() {
					Ok(Some(PathBuf::from(s)))
				} else {
					Err(CommonError::UiInteraction("Save dialog response not a string or null".into()))
				}
			},

			Ok(Ok(Err(e))) => Err(e),

			Ok(Err(_)) => {
				Err(CommonError::UiInteraction(format!(
					"Save dialog (ReqID: {}) channel closed.",
					request_id
				)))
			},

			Err(_) => {
				warn!("[Env UiProvider] show_save_dialog (ReqID: {}) timed out.", request_id);

				Ok(None)
			},
		};

		self.get_app_state()
			.pending_ui_requests
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.remove(&request_id);

		result
	}

	// QuickPick and InputBox follow the same pattern as
	// show_open_dialog/show_save_dialog
	async fn show_quick_pick(
		&self,

		items:Vec<QuickPickItem>,

		options:Option<QuickPickOptions>,
	) -> Result<Option<Vec<String>>, CommonError> {
		let request_id = Uuid::new_v4().to_string();

		info!(
			"[Env UiProvider] show_quick_pick (ReqID: {}): {} items, options={:?}",
			request_id,
			items.len(),
			options
		);

		let (tx, rx) = TokioOneshot::channel();

		{
			self.get_app_state()
				.pending_ui_requests
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?
				.insert(request_id.clone(), tx);
		}

		let serializable_items = items.into_iter().map(|item| json!({"label": item.label, "description": item.description, "detail": item.detail, "picked": item.picked, "alwaysShow": item.always_show })).collect::<Vec<_>>();

		let payload_data = json!({ "items": serializable_items, "options": options });

		let event_payload = UiRequestToSkyPayload { request_id:request_id.clone(), payload:payload_data };

		self.app_handle
			.emit_all("sky://ui/show-quick-pick-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed to emit show_quick_pick request: {}", e)))?;

		let result = match tokio_timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(v))) => {
				// Parse v into Option<Vec<String>> based on options.canPickMany
				if v.is_null() {
					Ok(None)
				} else if options.as_ref().map_or(false, |o| o.can_pick_many) {
					if let Some(arr) = v.as_array() {
						arr.iter()
							.map(|s_val| {
								s_val.as_str().map(String::from).ok_or_else(|| {
									CommonError::UiInteraction(
										"Invalid string in quick pick multi-select response".into(),
									)
								})
							})
							.collect::<Result<Vec<_>, _>>()
							.map(Some)
					} else {
						Err(CommonError::UiInteraction(
							"Quick pick (multi) response not an array or null".into(),
						))
					}
				} else {
					if let Some(s) = v.as_str() {
						Ok(Some(vec![s.to_string()]))
					} else {
						Err(CommonError::UiInteraction(
							"Quick pick (single) response not a string or null".into(),
						))
					}
				}
			},

			Ok(Ok(Err(e))) => Err(e),

			Ok(Err(_)) => {
				Err(CommonError::UiInteraction(format!(
					"Quick pick (ReqID: {}) channel closed.",
					request_id
				)))
			},

			Err(_) => {
				warn!("[Env UiProvider] show_quick_pick (ReqID: {}) timed out.", request_id);

				Ok(None)
			},
		};

		self.get_app_state()
			.pending_ui_requests
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.remove(&request_id);

		result
	}

	async fn show_input_box(&self, options:Option<InputBoxOptions>) -> Result<Option<String>, CommonError> {
		let request_id = Uuid::new_v4().to_string();

		info!("[Env UiProvider] show_input_box (ReqID: {}): options={:?}", request_id, options);

		let (tx, rx) = TokioOneshot::channel();

		{
			self.get_app_state()
				.pending_ui_requests
				.lock()
				.map_err(map_app_state_lock_error_to_common_error)?
				.insert(request_id.clone(), tx);
		}

		let event_payload = UiRequestToSkyPayload { request_id:request_id.clone(), payload:options.clone() };

		self.app_handle
			.emit_all("sky://ui/show-input-box-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed to emit show_input_box request: {}", e)))?;

		let result = match tokio_timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(v))) => {
				// Parse v into Option<String>
				if v.is_null() {
					Ok(None)
				} else if let Some(s) = v.as_str() {
					Ok(Some(s.to_string()))
				} else {
					Err(CommonError::UiInteraction("Input box response not a string or null".into()))
				}
			},

			Ok(Ok(Err(e))) => Err(e),

			Ok(Err(_)) => {
				Err(CommonError::UiInteraction(format!(
					"Input box (ReqID: {}) channel closed.",
					request_id
				)))
			},

			Err(_) => {
				warn!("[Env UiProvider] show_input_box (ReqID: {}) timed out.", request_id);

				Ok(None)
			},
		};

		self.get_app_state()
			.pending_ui_requests
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.remove(&request_id);

		result
	}
}

/// Helper struct for serializing UiProvider request payloads sent via Tauri
/// events to Sky.
#[derive(Serialize, Clone)]
struct UiRequestToSkyPayload<T:Serialize + Clone> {
	request_id:String,

	// Payload specific to the UI request type (e.g., OpenDialogOptions)
	payload:T,
}

impl Requires<Arc<dyn UiProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn UiProvider + Send + Sync> { Arc::new(self.clone()) }
}

// Skeletons for other providers, actual implementation would be detailed.
#[async_trait]
impl IpcProvider for MountainEnvironment {
	async fn send_notification_to_sidecar(
		&self,

		sidecar_id:String,

		method:String,

		params:Value,
	) -> Result<(), CommonError> {
		trace!(
			"[Env IpcProvider] Sending notification to sidecar '{}': method='{}'",
			sidecar_id, method
		);

		vine::send_notification_to_sidecar(&sidecar_id, method, params)
			.await
			.map_err(|vine_err| CommonError::IpcError(vine_err.to_string()))
	}

	async fn send_request_to_sidecar(
		&self,

		sidecar_id:String,

		method:String,

		params:Value,

		timeout_ms:u64,
	) -> Result<Value, CommonError> {
		trace!(
			"[Env IpcProvider] Sending request to sidecar '{}': method='{}'",
			sidecar_id, method
		);

		vine::send_request_to_sidecar(&sidecar_id, method, params, timeout_ms)
			.await
			.map_err(|vine_err| CommonError::IpcError(vine_err.to_string()))
	}
}

impl Requires<Arc<dyn IpcProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn IpcProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl LanguageFeatureProviderRegistry for MountainEnvironment {
	async fn register_provider(
		&self,

		sidecar_id:String,

		provider_type_common:CommonProviderType,

		selector_dto_val:Value,

		options_dto_val_opt:Option<Value>,
	) -> Result<u32, CommonError> {
		let app_state = self.get_app_state();

		let new_provider_handle = app_state.get_next_provider_handle();

		// Requires From impl
		let app_state_provider_type:AppStateLanguageProviderType = provider_type_common.into();

		info!(
			"[Env LangFeatRegistry Register] ProviderType='{:?}', Handle={}, SidecarID='{}', OptionsIsSome={}",
			app_state_provider_type,
			new_provider_handle,
			sidecar_id,
			options_dto_val_opt.is_some()
		);

		trace!(
			"[Env LangFeatRegistry Register] Selector DTO: {:?}, Options DTO: {:?}",
			selector_dto_val, options_dto_val_opt
		);

		// Parse options DTO for specific fields like triggerCharacters,

		// supportsResolveDetails, etc.
		let trigger_chars_opt:Option<Vec<String>> = options_dto_val_opt
			.as_ref()
			.and_then(|opts| opts.get("triggerCharacters"))
			.and_then(Value::as_array)
			.map(|arr| arr.iter().filter_map(Value::as_str).map(String::from).collect());

		let supports_resolve_opt:Option<bool> = options_dto_val_opt
			.as_ref()
			.and_then(|opts| opts.get("supportsResolveDetails"))
			.and_then(Value::as_bool);

		// ... and so on for other metadata like codeActionMetadata,

		// signatureHelpMetadata ...

		let new_registration = ProviderRegistration {
			handle:new_provider_handle,

			provider_type:app_state_provider_type,

			selector:selector_dto_val,

			sidecar_id,

			trigger_characters:trigger_chars_opt,

			supports_resolve_details:supports_resolve_opt,

			code_action_metadata:options_dto_val_opt.as_ref().and_then(|o| o.get("codeActionMetadata")).cloned(),

			signature_help_metadata:options_dto_val_opt
				.as_ref()
				.and_then(|o| o.get("signatureHelpMetadata"))
				.cloned(),
		};

		app_state
			.language_providers
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.insert(new_provider_handle, new_registration);

		Ok(new_provider_handle)
	}

	async fn unregister_provider(&self, provider_handle_to_remove:u32) -> Result<(), CommonError> {
		info!(
			"[Env LangFeatRegistry Unregister] Provider Handle: {}",
			provider_handle_to_remove
		);

		if self
			.get_app_state()
			.language_providers
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.remove(&provider_handle_to_remove)
			.is_none()
		{
			warn!(
				"[Env LangFeatRegistry Unregister] Attempted to unregister non-existent provider handle: {}",
				provider_handle_to_remove
			);
		}

		Ok(())
	}

	async fn get_providers_for_document(
		&self,

		document_uri:Url,

		language_id:String,

		provider_type_common:CommonProviderType,
	) -> Result<Vec<ProviderDescription>, CommonError> {
		debug!(
			"[Env LangFeatRegistry GetProviders] For Doc='{}...', Lang='{}', ProviderType='{:?}'",
			document_uri.path_segments().and_then(|s| s.last()).unwrap_or_default(),
			language_id,
			provider_type_common
		);

		let app_state_val = self.get_app_state();

		let providers_map_guard = app_state_val
			.language_providers
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		let target_app_state_provider_type:AppStateLanguageProviderType = provider_type_common.into();

		let mut matching_providers_vec = Vec::new();

		for registration_entry in providers_map_guard.values() {
			if registration_entry.provider_type == target_app_state_provider_type {
				// Use the config helper for document selector matching
				if handlers::config::match_document_selector(&registration_entry.selector, &document_uri, &language_id)
				{
					trace!(
						"[Env LangFeatRegistry GetProviders] Match found: Handle {}, Type {:?}, for Doc '{}', Lang \
						 '{}'",
						registration_entry.handle,
						registration_entry.provider_type,
						document_uri.as_str(),
						language_id
					);

					// Construct options Value from stored registration metadata
					let mut options_map_for_desc = JsonMap::new();

					if let Some(ref tc_vec) = registration_entry.trigger_characters {
						options_map_for_desc.insert("triggerCharacters".to_string(), json!(tc_vec));
					}

					if let Some(srd_bool) = registration_entry.supports_resolve_details {
						options_map_for_desc.insert("supportsResolveDetails".to_string(), json!(srd_bool));
					}

					if let Some(ref cam_val) = registration_entry.code_action_metadata {
						options_map_for_desc.insert("codeActionMetadata".to_string(), cam_val.clone());
					}

					if let Some(ref shm_val) = registration_entry.signature_help_metadata {
						options_map_for_desc.insert("signatureHelpMetadata".to_string(), shm_val.clone());
					}

					matching_providers_vec.push(ProviderDescription {
						handle:registration_entry.handle,

						sidecar_id:registration_entry.sidecar_id.clone(),

						options:if options_map_for_desc.is_empty() {
							None
						} else {
							Some(Value::Object(options_map_for_desc))
						},
					});
				}
			}
		}

		debug!(
			"[Env LangFeatRegistry GetProviders] Found {} matching {:?} providers for doc='{}...', lang='{}'",
			matching_providers_vec.len(),
			provider_type_common,
			document_uri.path_segments().and_then(|s| s.last()).unwrap_or_default(),
			language_id
		);

		Ok(matching_providers_vec)
	}
}

// Required From trait implementation for mapping common provider type to
// app_state specific type.
impl From<CommonProviderType> for AppStateLanguageProviderType {
	fn from(common_type:CommonProviderType) -> Self {
		match common_type {
			CommonProviderType::Hover => AppStateLanguageProviderType::Hover,

			CommonProviderType::Completion => AppStateLanguageProviderType::Completion,

			CommonProviderType::Definition => AppStateLanguageProviderType::Definition,

			CommonProviderType::Declaration => AppStateLanguageProviderType::Declaration,

			CommonProviderType::Implementation => AppStateLanguageProviderType::Implementation,

			CommonProviderType::TypeDefinition => AppStateLanguageProviderType::TypeDefinition,

			CommonProviderType::References => AppStateLanguageProviderType::References,

			CommonProviderType::DocumentHighlight => AppStateLanguageProviderType::DocumentHighlight,

			CommonProviderType::DocumentSymbol => AppStateLanguageProviderType::DocumentSymbol,

			CommonProviderType::WorkspaceSymbol => AppStateLanguageProviderType::WorkspaceSymbol,

			CommonProviderType::CodeAction => AppStateLanguageProviderType::CodeAction,

			CommonProviderType::CodeLens => AppStateLanguageProviderType::CodeLens,

			CommonProviderType::Formatting => AppStateLanguageProviderType::Formatting,

			CommonProviderType::RangeFormatting => AppStateLanguageProviderType::RangeFormatting,

			CommonProviderType::OnTypeFormatting => AppStateLanguageProviderType::OnTypeFormatting,

			CommonProviderType::Rename => AppStateLanguageProviderType::Rename,

			CommonProviderType::DocumentLink => AppStateLanguageProviderType::DocumentLink,

			CommonProviderType::Color => AppStateLanguageProviderType::Color,

			CommonProviderType::FoldingRange => AppStateLanguageProviderType::FoldingRange,

			CommonProviderType::SelectionRange => AppStateLanguageProviderType::SelectionRange,

			CommonProviderType::CallHierarchy => AppStateLanguageProviderType::CallHierarchy,

			CommonProviderType::TypeHierarchy => AppStateLanguageProviderType::TypeHierarchy,

			CommonProviderType::LinkedEditingRange => AppStateLanguageProviderType::LinkedEditingRange,

			CommonProviderType::InlayHints => AppStateLanguageProviderType::InlayHints,
		}
	}
}

impl Requires<Arc<dyn LanguageFeatureProviderRegistry + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn LanguageFeatureProviderRegistry + Send + Sync> { Arc::new(self.clone()) }
}

// Stubs for CommandExecutor and WorkspaceProvider, assuming their logic is
// complex and would primarily delegate to `handlers::commands` and
// `handlers::workspace` or `AppState` methods.
#[async_trait]
impl CommandExecutor for MountainEnvironment {
	async fn execute_command(&self, command_id:String, args_val:Value) -> Result<Value, CommonError> {
		info!("[Env CommandExecutor] Execute: command_id='{}'", command_id);

		trace!("[Env CommandExecutor] Args: {:?}", args_val);

		let main_window = self
			.app_handle
			.get_window("main")
			.ok_or_else(|| CommonError::UiInteraction("Main window not found for command execution".to_string()))?;

		let app_runtime_state = self.app_handle.state::<Arc<AppRuntime>>();

		handlers::commands::handle_execute_command(
			self.app_handle.clone(),
			main_window,
			// Pass the Arc<AppRuntime>
			app_runtime_state.inner().clone(),
			json!({ "id": command_id, "args": args_val }),
		)
		.await
		.map_err(|json_rpc_err_str| CommonError::CommandExecution(command_id, json_rpc_err_str))
	}

	async fn register_command(&self, sidecar_id:String, command_id:String) -> Result<(), CommonError> {
		info!(
			"[Env CommandExecutor] Register: sidecar_id='{}', command_id='{}'",
			sidecar_id, command_id
		);

		handlers::commands::handle_register_command(self.app_handle.clone(), sidecar_id, json!({ "id": command_id }))
			.await
			.map(|_| ())
			.map_err(|e| CommonError::CommandRegistration(command_id, e))
	}

	async fn unregister_command(&self, sidecar_id:String, command_id:String) -> Result<(), CommonError> {
		info!(
			"[Env CommandExecutor] Unregister: sidecar_id='{}', command_id='{}'",
			sidecar_id, command_id
		);

		handlers::commands::handle_unregister_command(self.app_handle.clone(), sidecar_id, json!({ "id": command_id }))
			.await
			.map(|_| ())
			.map_err(|e| CommonError::CommandRegistration(command_id, e))
	}

	async fn get_all_commands(&self) -> Result<Vec<String>, CommonError> {
		debug!("[Env CommandExecutor] GetAllCommands");

		let app_runtime_state = self.app_handle.state::<Arc<AppRuntime>>();

		handlers::commands::handle_get_commands(self.app_handle.clone(), app_runtime_state.inner().clone())
			.await
			.and_then(|val| serde_json::from_value(val).map_err(|e_serde| e_serde.to_string()))
			.map_err(CommonError::CommandList)
	}
}

impl Requires<Arc<dyn CommandExecutor + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn CommandExecutor + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl WorkspaceProvider for MountainEnvironment {
	async fn get_workspace_folders_info(&self) -> Result<Vec<(Url, String, usize)>, CommonError> {
		trace!("[Env WorkspaceProvider] GetWorkspaceFoldersInfo");

		let app_state = self.get_app_state();

		let folders_guard = app_state
			.workspace_folders
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		Ok(folders_guard
			.iter()
			.map(|f_state| (f_state.uri.clone(), f_state.name.clone(), f_state.index))
			.collect())
	}

	// ... other WorkspaceProvider methods similarly delegating or using AppState
	// ...
	async fn get_workspace_folder_info(&self, uri_to_match:Url) -> Result<Option<(Url, String, usize)>, CommonError> {
		debug!(
			"[Env WorkspaceProvider] Getting specific workspace folder info for: {}",
			uri_to_match
		);

		let app_state = self.get_app_state();

		let folders_guard = app_state
			.workspace_folders
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		Ok(folders_guard.iter()

			 // Basic prefix match
			.find(|f_state| uri_to_match.as_str().starts_with(f_state.uri.as_str()))

			.map(|f_state| (f_state.uri.clone(), f_state.name.clone(), f_state.index)))
	}

	async fn get_workspace_name(&self) -> Result<Option<String>, CommonError> {
		debug!("[Env WorkspaceProvider] Getting workspace name");

		self.get_app_state()
			.get_workspace_name()
			.map(Some)
			.map_err(CommonError::StateLock)
	}

	async fn get_workspace_configuration_path(&self) -> Result<Option<PathBuf>, CommonError> {
		debug!("[Env WorkspaceProvider] Getting workspace config path");

		Ok(self
			.get_app_state()
			.workspace_config_path
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.clone())
	}

	async fn is_workspace_trusted(&self) -> Result<bool, CommonError> {
		debug!("[Env WorkspaceProvider] Getting workspace trust state");

		Ok(self.get_app_state().is_trusted.load(std::sync::atomic::Ordering::Relaxed))
	}

	async fn request_workspace_trust(&self, _options:Option<Value>) -> Result<bool, CommonError> {
		info!("[Env WorkspaceProvider] Requesting workspace trust (options: {:?})", _options);

		warn!(
			"[Env WorkspaceProvider] requestWorkspaceTrust is STUBBED to return current trust state. Full UI \
			 interaction flow needed."
		);

		// TODO: Full implementation should use UiProvider to show a dialog if not
		// trusted.
		Ok(self.get_app_state().is_trusted.load(std::sync::atomic::Ordering::Relaxed))
	}

	async fn find_files_in_workspace(
		&self,

		include_pattern_dto:Value,

		exclude_pattern_dto_opt:Option<Value>,

		max_results_opt:Option<usize>,

		use_ignore_files_bool:bool,

		follow_symlinks_bool:bool,
	) -> Result<Vec<Url>, CommonError> {
		info!(
			"[Env WorkspaceProvider] Finding files: include='{:?}', exclude='{:?}'",
			include_pattern_dto, exclude_pattern_dto_opt
		);

		// Construct params array for handlers::workspace::handle_find_files
		let params_for_handler = json!([
			include_pattern_dto,


			exclude_pattern_dto_opt.unwrap_or(Value::Null),


			{ "maxResults": max_results_opt, "useIgnoreFiles": use_ignore_files_bool, "followSymlinks": follow_symlinks_bool }



		]);

		handlers::workspace::handle_find_files(self.app_handle.clone(), params_for_handler)
			.await
			.and_then(|uri_components_array_val| {
				uri_components_array_val.as_array().map_or_else(
					|| Err(CommonError::Unknown("findFiles handler did not return an array".to_string())),
					|uri_dtos_vec| {
						uri_dtos_vec
							.iter()
							.map(|uri_comp_dto| {
								let uri_str =
									uri_comp_dto.get("external").and_then(Value::as_str).ok_or_else(|| {
										CommonError::Unknown(
											"Invalid URI component DTO in findFiles result (missing 'external')"
												.to_string(),
										)
									})?;

								Url::parse(uri_str).map_err(|e_url_parse| {
									CommonError::Unknown(format!(
										"Failed to parse URI from findFiles result ('{}'): {}",
										uri_str, e_url_parse
									))
								})
							})
							.collect::<Result<Vec<Url>, CommonError>>()
					},
				)
			})
	}
}

impl Requires<Arc<dyn WorkspaceProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn WorkspaceProvider + Send + Sync> { Arc::new(self.clone()) }
}
