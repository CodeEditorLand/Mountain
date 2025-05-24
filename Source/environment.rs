// ---------------------------------------------------------------------------------------------
// Mountain Environment Implementation (environment.rs)
// --------------------------------------------------------------------------------------------
// Defines `MountainEnvironment`, the concrete implementation of the abstract
// `Environment` trait and various provider traits. This struct provides
// the actual "native" implementations for `ActionEffect`s.
//
// Responsibilities:
// - Implementing all provider traits.
// - Filesystem access: Using `tokio::fs`, performing security checks.
// - Configuration:
//   - Accessing `AppState.configuration` (the merged view) for reads.
//   - Using `handlers::config` helpers for file I/O for writes, JSON
//     manipulation, re-merging global state, and notifying Cocoon of changes.
// - Document state: Managing `DocumentState` in `AppState`, applying changes
//   via `DocumentState::apply_changes`, calling
//   `handlers::documents::notify_...`.
// - Storage, Secrets, Diagnostics, Commands, Output: Delegating or managing
//   state.
// - Language Features: Storing provider registrations, basic (stubbed) provider
//   retrieval.
// - UI Interactions: Basic messages via Tauri dialogs; detailed stubs for
//   complex UI (dialogs with return values, quick picks, input boxes) outlining
//   event flow with Sky.
// - Holding `AppHandle` for state/API access.
// - Mapping errors to `CommonError`.
//
// Key Interactions:
// - Instantiated in `main.rs`, held by `AppRuntime`.
// - Methods called by `AppRuntime::run` for effects.
// - Accesses `AppState` via `self.get_app_state()`.
// - Uses `handlers::config` for all configuration persistence and update logic.
// - Calls `handlers::documents::notify_...` after modifying document state.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	ffi::OsStr,

	path::{Path, PathBuf},

	sync::{Arc, Mutex as StdMutex, MutexGuard},

	// Renamed to avoid conflict if tokio::time::Duration is also used
	time::Duration as StdDuration,
};

use Land_Common::{
	command_effects::CommandExecutor,

	config_effects::{
		ConfigInspector,

		ConfigProvider,

		ConfigurationScope,

		ConfigurationTarget,

		IConfigurationOverrides,

		// For ConfigInspector
		InspectResultData,
	},

	diagnostics_effects::DiagnosticsManager,

	documents_effects::{DocEventParams, DocumentProvider},

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
use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
// Added Serialize for UiProvider request DTOs
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window, Wry};
use tokio::{
	fs,

	io::AsyncWriteExt,

	// Added oneshot for UiProvider async responses
	sync::oneshot,

	// For UI interaction timeouts
	time::{Duration as TokioDuration, timeout},
};
use url::Url;
// For generating unique request IDs for UI interactions
use uuid::Uuid;

use crate::{
	app_state::{
		self,
		AppState,
		ConfigurationState,
		DocumentState,
		LanguageProviderMap,
		LanguageProviderType,
		OutputChannelState,
		ProviderRegistration,
		StorageMap,
		WorkspaceFolderState,
	},
	handlers,
	runtime::AppRuntime,
	vine,
};

// --- Mountain Environment Struct ---
#[derive(Clone)]
pub struct MountainEnvironment {
	app_handle:AppHandle<Wry>,
}

impl MountainEnvironment {
	pub fn new(app_handle:AppHandle<Wry>) -> Self {
		info!("[Env] MountainEnvironment instance created.");

		Self { app_handle }
	}

	fn get_app_state(&self) -> State<'_, AppState> { self.app_handle.state::<AppState>() }

	async fn is_path_allowed(&self, path:&Path) -> Result<(), CommonError> {
		trace!("[Env Security] Checking path allowance for: {}", path.display());

		let path_owned = path.to_path_buf();

		let canonical_path_res = tokio::task::spawn_blocking(move || -> Result<PathBuf, std::io::Error> {
			match std::fs::canonicalize(&path_owned) {
				Ok(p) => Ok(p),

				Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
					path_owned
						.parent()
						.map_or_else(
							|| Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Path has no parent")),
							std::fs::canonicalize,
						)
						.map(|p| p.join(path_owned.file_name().unwrap_or_default()))
				},

				Err(e) => Err(e),
			}
		})
		.await;

		let canonical_path = match canonical_path_res {
			Ok(Ok(p)) => p,

			Ok(Err(e)) => {
				return Err(CommonError::FsPermissionDenied(
					path.to_path_buf(),
					format!("Path canonicalization failed: {}", e),
				));
			},

			Err(e) => {
				return Err(CommonError::FsPermissionDenied(
					path.to_path_buf(),
					format!("Task join error during canonicalization: {}", e),
				));
			},
		};

		trace!(
			"[Env Security] Canonical path for '{}': '{}'",
			path.display(),
			canonical_path.display()
		);

		let mut allowed_roots:Vec<PathBuf> = Vec::new();

		let state = self.get_app_state();

		let folders_guard = state.workspace_folders.lock().map_err(map_lock_error)?;

		for folder in folders_guard.iter() {
			if folder.uri.scheme() == "file" {
				if let Ok(p) = std::fs::canonicalize(PathBuf::from(folder.uri.path())) {
					allowed_roots.push(p);
				} else {
					warn!("[Env Security] Failed to canonicalize workspace folder URI: {}", folder.uri);
				}
			} else {
				warn!("[Env Security] Non-file scheme for workspace folder, skipping: {}", folder.uri);
			}
		}

		drop(folders_guard);

		let path_resolver = self.app_handle.path_resolver();

		for dir_opt in [
			path_resolver.app_config_dir(),
			path_resolver.app_data_dir(),
			path_resolver.app_log_dir(),
		] {
			if let Some(dir) = dir_opt {
				if let Ok(p) = std::fs::canonicalize(&dir) {
					allowed_roots.push(p);
				} else {
					warn!("[Env Security] Failed to canonicalize app system dir: {}", dir.display());
				}
			}
		}

		if let Ok(p) = std::fs::canonicalize(&state.global_memento_path) {
			allowed_roots.push(p);
		}

		if let Some(ref wsp_path_opt) = *state.workspace_memento_path.lock().map_err(map_lock_error)? {
			// Dereference Arc<Mutex<Option<PathBuf>>>
			if let Some(ref wsp_path) = wsp_path_opt {
				// Check Option<PathBuf>
				if let Ok(p) = std::fs::canonicalize(wsp_path) {
					allowed_roots.push(p);
				}
			}
		}

		let is_allowed = allowed_roots
			.iter()
			.any(|root| canonical_path == *root || canonical_path.starts_with(root));

		if is_allowed {
			trace!("[Env Security] ALLOWED: '{}'", path.display());

			Ok(())
		} else {
			warn!(
				"[Env Security] DENIED: '{}' (canonical: '{}'). Not in roots: {:?}",
				path.display(),
				canonical_path.display(),
				allowed_roots
			);

			Err(CommonError::FsPermissionDenied(
				path.to_path_buf(),
				"Path outside allowed workspace/app data folders.".to_string(),
			))
		}
	}
}

impl Environment for MountainEnvironment {}

// --- Helper Error/Util Functions ---
fn map_lock_error<T>(e:std::sync::PoisonError<MutexGuard<'_, T>>) -> CommonError {
	CommonError::StateLock(format!("Failed to lock AppState section: {}", e))
}

fn map_io_error_to_common(e:std::io::Error, path:PathBuf, operation:&'static str) -> CommonError {
	warn!(
		"[Env IO Error] Operation '{}' on path '{}' failed: {}",
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

		std::io::ErrorKind::DirectoryNotEmpty => CommonError::FsNotEmpty(path),

		_ => {
			match operation {
				"read" => CommonError::FsRead(path, e.to_string()),

				"write" => CommonError::FsWrite(path, e.to_string()),

				"stat" => CommonError::FsStat(path, e.to_string()),

				"readdir" | "readdir_next" => CommonError::FsReadDir(path, e.to_string()),

				"mkdir" | "mkdir_parent" | "mkdir_all" | "mkdir_parent_rename" | "mkdir_parent_copy" => {
					CommonError::FsMkdir(path, e.to_string())
				},

				"delete" | "delete_stat_check" => CommonError::FsDelete(path, e.to_string()),

				"rename" | "rename_target_stat" => CommonError::FsRename(path, e.to_string()),

				"copy" | "copy_source_stat" => CommonError::FsCopy(path, e.to_string()),

				"read_doc_open" => CommonError::FsRead(path, format!("Failed to read document for opening: {}", e)),

				"write_doc_save" => CommonError::FsWrite(path, format!("Failed to write document for saving: {}", e)),

				"write_doc_save_as" => {
					CommonError::FsWrite(path, format!("Failed to write document for save_as: {}", e))
				},

				_ => CommonError::Unknown(format!("FS Op '{}' on '{}' failed: {}", operation, path.display(), e)),
			}
		},
	}
}

fn detect_language_id(path:&Path) -> String {
	match path.extension().and_then(OsStr::to_str) {
		Some("js") | Some("mjs") | Some("cjs") => "javascript",

		Some("jsx") => "javascriptreact",

		Some("ts") => "typescript",

		Some("tsx") => "typescriptreact",

		Some("json") => "json",

		Some("jsonc") => "jsonc",

		Some("html") | Some("htm") => "html",

		Some("css") => "css",

		Some("scss") => "scss",

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

		Some("kt") => "kotlin",

		Some("dart") => "dart",

		Some("lua") => "lua",

		Some("sql") => "sql",

		Some("ps1") => "powershell",

		Some("bat") | Some("cmd") => "bat",

		_ => "plaintext",
	}
	.to_string()
}

// Simplified for MVP
fn detect_encoding(_content:&[u8]) -> String { "utf8".to_string() }

// --- Effect Provider Trait Implementations ---

#[async_trait]
impl FsReader for MountainEnvironment {
	async fn read_file(&self, path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		self.is_path_allowed(path).await?;

		trace!("[Env FsReader] Reading file: {}", path.display());

		fs::read(path)
			.await
			.map_err(|e| map_io_error_to_common(e, path.clone(), "read"))
	}

	async fn stat_file(&self, path:&PathBuf) -> Result<FileSystemStat, CommonError> {
		self.is_path_allowed(path).await?;

		trace!("[Env FsReader] Stating file: {}", path.display());

		match tokio::fs::metadata(path).await {
			Ok(metadata) => {
				let mut file_type_val = 0;

				if metadata.is_file() {
					file_type_val |= CommonFileType::File as u8;
				}

				if metadata.is_dir() {
					file_type_val |= CommonFileType::Directory as u8;
				}

				if metadata.is_symlink() {
					file_type_val |= CommonFileType::SymbolicLink as u8;
				}

				if file_type_val == 0 {
					file_type_val = CommonFileType::Unknown as u8;
				}

				let get_milli_ts = |st:Result<_, _>| -> u64 {
					st.ok()
						.and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
						.map_or(0, |d| d.as_millis() as u64)
				};

				Ok(FileSystemStat {
					file_type:file_type_val,

					ctime:get_milli_ts(metadata.created()),

					mtime:get_milli_ts(metadata.modified()),

					size:metadata.len(),

					// TODO: Populate permissions if needed by VS Code extensions
					permissions:None,
				})
			},

			Err(e) => Err(map_io_error_to_common(e, path.clone(), "stat")),
		}
	}

	async fn read_directory(&self, path:&PathBuf) -> Result<Vec<(String, CommonFileType)>, CommonError> {
		self.is_path_allowed(path).await?;

		debug!("[Env FsReader] Reading directory: {}", path.display());

		let mut entries_vec:Vec<(String, CommonFileType)> = Vec::new();

		let mut read_dir = fs::read_dir(path)
			.await
			.map_err(|e| map_io_error_to_common(e, path.clone(), "readdir"))?;

		while let Some(entry_res) = read_dir
			.next_entry()
			.await
			.map_err(|e| map_io_error_to_common(e, path.clone(), "readdir_next"))?
		{
			let file_name = entry_res.file_name().to_string_lossy().into_owned();

			match entry_res.file_type().await {
				Ok(ft) => {
					let common_ft = if ft.is_dir() {
						CommonFileType::Directory
					} else if ft.is_file() {
						CommonFileType::File
					} else if ft.is_symlink() {
						CommonFileType::SymbolicLink
					} else {
						CommonFileType::Unknown
					};

					entries_vec.push((file_name, common_ft));
				},

				Err(e) => {
					warn!(
						"[Env FsReader] Failed to get type for entry '{}' in '{}': {}",
						file_name,
						path.display(),
						e
					);

					entries_vec.push((file_name, CommonFileType::Unknown));
				},
			}
		}

		Ok(entries_vec)
	}
}

impl Requires<Arc<dyn FsReader + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn FsReader + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl FsWriter for MountainEnvironment {
	async fn write_file(&self, path:&PathBuf, content:Vec<u8>, create:bool, overwrite:bool) -> Result<(), CommonError> {
		self.is_path_allowed(path).await?;

		info!(
			"[Env FsWriter] Writing file: {} ({} bytes), create={}, overwrite={}",
			path.display(),
			content.len(),
			create,
			overwrite
		);

		let path_exists = fs::try_exists(path).await.unwrap_or(false);

		if path_exists && !overwrite {
			return Err(CommonError::FsFileExists(path.clone()));
		}

		if !path_exists && !create {
			return Err(CommonError::FsNotFound(path.clone()));
		}

		if let Some(p_dir) = path.parent() {
			if !fs::try_exists(p_dir).await.unwrap_or(false) {
				if create {
					fs::create_dir_all(p_dir)
						.await
						.map_err(|e| map_io_error_to_common(e, p_dir.to_path_buf(), "mkdir_parent"))?;
				} else {
					return Err(CommonError::FsNotFound(p_dir.to_path_buf()));
				}
			}
		}

		fs::write(path, &content)
			.await
			.map_err(|e| map_io_error_to_common(e, path.clone(), "write"))?;

		// TODO: Emit filesystem_changed event via AppHandle if this direct write
		// bypasses higher-level logic that would do so.
		Ok(())
	}

	async fn create_directory(&self, path:&PathBuf, recursive:bool) -> Result<(), CommonError> {
		self.is_path_allowed(path).await?;

		info!(
			"[Env FsWriter] Creating directory: {} (recursive={})",
			path.display(),
			recursive
		);

		if recursive {
			fs::create_dir_all(path)
				.await
				.map_err(|e| map_io_error_to_common(e, path.clone(), "mkdir_all"))?;
		} else {
			fs::create_dir(path)
				.await
				.map_err(|e| map_io_error_to_common(e, path.clone(), "mkdir"))?;
		}

		// TODO: Emit filesystem_changed event
		Ok(())
	}

	async fn delete(&self, path:&PathBuf, recursive:bool, use_trash:bool) -> Result<(), CommonError> {
		self.is_path_allowed(path).await?;

		info!(
			"[Env FsWriter] Deleting: {} (recursive={}, useTrash={})",
			path.display(),
			recursive,
			use_trash
		);

		if use_trash {
			warn!("[Env FsWriter] 'useTrash' option for delete is not yet implemented, using permanent delete.");

			// TODO: Implement trash functionality using libraries like `trash`.
		}

		match fs::metadata(path).await {
			Ok(md) => {
				let del_op = if md.is_dir() {
					if recursive {
						fs::remove_dir_all(path).await
					} else {
						fs::remove_dir(path).await
					}
				} else {
					fs::remove_file(path).await
				};

				del_op.map_err(|e| map_io_error_to_common(e, path.clone(), "delete"))?;

				// TODO: Emit filesystem_changed event
				Ok(())
			},

			// Deleting non-existent is OK
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),

			Err(e) => Err(map_io_error_to_common(e, path.clone(), "delete_stat_check")),
		}
	}

	async fn rename(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		self.is_path_allowed(source).await?;

		self.is_path_allowed(target).await?;

		info!(
			"[Env FsWriter] Renaming: {} -> {} (overwrite={})",
			source.display(),
			target.display(),
			overwrite
		);

		if !fs::try_exists(source).await.unwrap_or(false) {
			return Err(CommonError::FsNotFound(source.clone()));
		}

		if !overwrite && fs::try_exists(target).await.unwrap_or(false) {
			return Err(CommonError::FsFileExists(target.clone()));
		}

		if overwrite && fs::try_exists(target).await.unwrap_or(false) {
			debug!(
				"[Env FsWriter] Rename: Overwriting target by deleting first: {}",
				target.display()
			);

			let target_meta = fs::metadata(target)
				.await
				.map_err(|e| map_io_error_to_common(e, target.clone(), "rename_target_stat"))?;

			self.delete(target, target_meta.is_dir(), false).await?;
		}

		if let Some(p_dir) = target.parent() {
			if !fs::try_exists(p_dir).await.unwrap_or(false) {
				fs::create_dir_all(p_dir)
					.await
					.map_err(|e| map_io_error_to_common(e, p_dir.to_path_buf(), "mkdir_parent_rename"))?;
			}
		}

		fs::rename(source, target)
			.await
			.map_err(|e| map_io_error_to_common(e, source.clone(), "rename"))?;

		// TODO: Emit filesystem_changed event (one delete for source, one create for
		// target, or a specific rename event)
		Ok(())
	}

	async fn copy(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		self.is_path_allowed(source).await?;

		self.is_path_allowed(target).await?;

		info!(
			"[Env FsWriter] Copying: {} -> {} (overwrite={})",
			source.display(),
			target.display(),
			overwrite
		);

		if !fs::try_exists(source).await.unwrap_or(false) {
			return Err(CommonError::FsNotFound(source.clone()));
		}

		if !overwrite && fs::try_exists(target).await.unwrap_or(false) {
			return Err(CommonError::FsFileExists(target.clone()));
		}

		let source_meta = fs::metadata(source)
			.await
			.map_err(|e| map_io_error_to_common(e, source.clone(), "copy_source_stat"))?;

		if source_meta.is_dir() {
			error!(
				"[Env FsWriter] Recursive directory copy not yet implemented for source: {}",
				source.display()
			);

			return Err(CommonError::NotImplemented("Recursive directory copy".to_string()));
		}

		if overwrite && fs::try_exists(target).await.unwrap_or(false) {
			debug!(
				"[Env FsWriter] Copy: Overwriting target by deleting first: {}",
				target.display()
			);

			self.delete(target, false, false).await?;
		}

		if let Some(p_dir) = target.parent() {
			if !fs::try_exists(p_dir).await.unwrap_or(false) {
				fs::create_dir_all(p_dir)
					.await
					.map_err(|e| map_io_error_to_common(e, p_dir.to_path_buf(), "mkdir_parent_copy"))?;
			}
		}

		fs::copy(source, target)
			.await
			.map(|_| ())
			.map_err(|e| map_io_error_to_common(e, source.clone(), "copy"))?;

		// TODO: Emit filesystem_changed event for target creation
		Ok(())
	}
}

impl Requires<Arc<dyn FsWriter + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn FsWriter + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl StorageProvider for MountainEnvironment {
	async fn get_storage_value(&self, scope_is_global:bool, key:&str) -> Result<Option<Value>, CommonError> {
		trace!(
			"[Env StorageProvider] Getting value: scope_global={}, key='{}'",
			scope_is_global, key
		);

		let app_state = self.get_app_state();

		let (storage_mutex, _path_opt) =
			handlers::storage::get_storage_map_and_path(&app_state, if scope_is_global { 1 } else { 0 })
				.map_err(|e_str| CommonError::StateLock(e_str))?;

		let storage_guard = storage_mutex.lock().map_err(map_lock_error)?;

		let value = storage_guard.get(key).cloned();

		debug!(
			"[Env StorageProvider] Value for key '{}' (scope_global={}): {:?}",
			key,
			scope_is_global,
			value.is_some()
		);

		Ok(value)
	}

	async fn update_storage_value(
		&self,

		scope_is_global:bool,

		key:String,

		value:Option<Value>,
	) -> Result<(), CommonError> {
		info!(
			"[Env StorageProvider] Updating value: scope_global={}, key='{}', has_value={}",
			scope_is_global,
			key,
			value.is_some()
		);

		let app_state = self.get_app_state();

		let (storage_mutex, path_opt) =
			handlers::storage::get_storage_map_and_path(&app_state, if scope_is_global { 1 } else { 0 })
				.map_err(|e_str| CommonError::StateLock(e_str))?;

		let data_to_save = {
			let mut storage_guard = storage_mutex.lock().map_err(map_lock_error)?;

			if let Some(val_to_set) = value {
				storage_guard.insert(key.clone(), val_to_set);
			} else {
				storage_guard.remove(&key);
			}

			path_opt.as_ref().map(|_| storage_guard.clone())
		};

		if let (Some(path), Some(data_clone)) = (path_opt, data_to_save) {
			debug!("[Env StorageProvider] Persisting storage to {}", path.display());

			tokio::spawn(async move {
				if let Err(e) = handlers::storage::save_storage_to_disk(&path, &data_clone).await {
					error!("[Env StorageProvider] Error persisting storage to {}: {}", path.display(), e);
				}
			});
		} else if !scope_is_global && path_opt.is_none() {
			warn!(
				"[Env StorageProvider] Workspace storage path not set. Cannot persist value for key '{}'.",
				key
			);
		}

		Ok(())
	}
}

impl Requires<Arc<dyn StorageProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn StorageProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl ConfigProvider for MountainEnvironment {
	async fn get_configuration_value(
		&self,

		section:Option<String>,

		overrides:IConfigurationOverrides,
	) -> Result<Value, CommonError> {
		trace!(
			"[Env ConfigProvider] Getting config: section={:?}, overrides.resource={:?}, overrides.langId={:?}",
			section,
			overrides.resource.as_ref().and_then(|v| v.get("external")),
			overrides.override_identifier
		);

		let app_state = self.get_app_state();

		let config_guard = app_state.configuration.lock().map_err(map_lock_error)?;

		if overrides.resource.is_some() || overrides.override_identifier.is_some() {
			warn!(
				"[Env ConfigProvider] get_configuration_value: Overrides provided but MVP implementation uses \
				 simplified lookup from merged state. Resource/language specific overrides from different files may \
				 not be fully resolved here beyond what `load_and_merge_configurations_internal` does."
			);
		}

		let value = config_guard.get_value(section.as_deref(), overrides.resource.as_ref());

		debug!(
			"[Env ConfigProvider] Value for section {:?}: {}...",
			section,
			value.to_string().chars().take(70).collect::<String>()
		);

		Ok(value)
	}

	async fn update_configuration_value(
		&self,

		key:String,

		value_to_set:Value,

		target:ConfigurationTarget,

		overrides:IConfigurationOverrides,

		scope_to_language:Option<bool>,
	) -> Result<(), CommonError> {
		info!(
			"[Env ConfigProvider] Updating config: key='{}', target={:?}, has_value={}, scope_to_lang={:?}, \
			 override_res={:?}",
			key,
			target,
			!value_to_set.is_null(),
			scope_to_language,
			overrides.resource.as_ref().and_then(|v| v.get("external"))
		);

		let app_state = self.get_app_state();

		let config_file_path = handlers::config::get_config_path_for_target(
			&self.app_handle,
			&app_state,
			target,
			&overrides,
			scope_to_language.unwrap_or(false),
		)?;

		info!(
			"[Env ConfigProvider] Target config file for update: {}",
			config_file_path.display()
		);

		let mut current_file_json = handlers::config::load_json_file_if_exists_or_default(&config_file_path).await?;

		trace!(
			"[Env ConfigProvider] Loaded JSON ({} keys) from {}",
			current_file_json.as_object().map_or(0, |m| m.keys().len()),
			config_file_path.display()
		);

		let mut effective_json_target_in_file = &mut current_file_json;

		let mut lang_key_holder:Option<String> = None;

		if scope_to_language.unwrap_or(false) {
			if let Some(lang_id) = &overrides.override_identifier {
				lang_key_holder = Some(format!("[{}]", lang_id));

				let lang_key_str = lang_key_holder.as_ref().unwrap();

				if !effective_json_target_in_file.is_object() {
					*effective_json_target_in_file = json!({});
				}

				effective_json_target_in_file = effective_json_target_in_file
					.entry(lang_key_str.clone())
					.or_insert_with(|| json!({}));
			} else {
				warn!(
					"[Env ConfigProvider] scopeToLanguage is true for key '{}', but no languageId in overrides. \
					 Updating at top level of specified file.",
					key
				);
			}
		}

		handlers::config::update_json_value_at_path(effective_json_target_in_file, &key, value_to_set);

		trace!(
			"[Env ConfigProvider] Key '{}' updated in in-memory JSON for file {}.",
			key,
			config_file_path.display()
		);

		handlers::config::write_json_file(&config_file_path, current_file_json).await?;

		info!(
			"[Env ConfigProvider] Successfully wrote updated config to {}",
			config_file_path.display()
		);

		let new_merged_state =
			handlers::config::load_and_merge_configurations_internal(&self.app_handle, &app_state).await?;

		app_state
			.configuration
			.lock()
			.map_err(map_lock_error)?
			.update_from(new_merged_state);

		info!(
			"[Env ConfigProvider] In-memory AppState.configuration reloaded and updated after change to {}",
			config_file_path.display()
		);

		handlers::config::notify_config_changed_for_keys(&self.app_handle, vec![key]).await;

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
			"[Env ConfigInspector] Inspecting key='{}', overrides.resource={:?}",
			key,
			overrides.resource.as_ref().and_then(|v| v.get("external"))
		);

		warn!("[Env ConfigInspector] inspect_configuration_value is STUBBED for MVP. Returning None.");

		// TODO: Implement full inspection logic by checking values in User, Workspace,

		// Folder settings files.
		Ok(None)
	}
}

impl Requires<Arc<dyn ConfigInspector + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn ConfigInspector + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl DocumentProvider for MountainEnvironment {
	async fn open_document(
		&self,

		uri_components:Value,

		language_id_opt:Option<String>,

		content_opt:Option<String>,
	) -> Result<Url, CommonError> {
		info!(
			"[Env DocumentProvider] Effect open_document: uri_components='{:?}', lang='{:?}', has_content={}",
			uri_components.get("external").or_else(|| uri_components.get("path")),
			language_id_opt,
			content_opt.is_some()
		);

		let uri_str = uri_components
			.get("external")
			.and_then(Value::as_str)
			.or_else(|| {
				uri_components.get("path").and_then(Value::as_str).map(|p| {
					if Path::new(p).is_absolute() {
						Url::from_file_path(p).map(|u| u.to_string()).unwrap_or_else(|_| p.to_string())
					} else {
						p.to_string()
					}
				})
			})
			.ok_or_else(|| {
				CommonError::InvalidArg("uri_components".to_string(), "Missing external or path string".to_string())
			})?;

		let uri = Url::parse(uri_str)
			.map_err(|e| CommonError::InvalidArg("uri".to_string(), format!("Invalid URI '{}': {}", uri_str, e)))?;

		let state = self.get_app_state();

		{
			let open_docs_guard = state.open_documents.lock().map_err(map_lock_error)?;

			if let Some(existing_doc) = open_docs_guard.get(uri.as_str()) {
				info!(
					"[Env DocumentProvider] Document {} already open with version {}. Returning existing.",
					uri, existing_doc.version
				);

				return Ok(uri);
			}
		}

		let (lines, eol, encoding, actual_language_id, initial_dirty_state) = if let Some(content) = content_opt {
			let (l, e) = app_state::lines_and_eol_from_text(&content);

			info!(
				"[Env DocumentProvider] Opening untitled document {} with provided content.",
				uri
			);

			(
				l,
				e,
				"utf8".to_string(),
				language_id_opt.unwrap_or_else(|| detect_language_id(Path::new(uri.path()))),
				true,
			)
		} else {
			if uri.scheme() != "file" {
				error!("[Env DocumentProvider] Attempted to open non-file URI {} without content.", uri);

				return Err(CommonError::NotImplemented(format!(
					"Opening non-file URIs without content: {}",
					uri.scheme()
				)));
			}

			let path = PathBuf::from(uri.path());

			self.is_path_allowed(&path).await?;

			debug!("[Env DocumentProvider] Reading file content for {}", path.display());

			let bytes = fs::read(&path)
				.await
				.map_err(|e| map_io_error_to_common(e, path.clone(), "read_doc_open"))?;

			let encoding_detected = detect_encoding(&bytes);

			let content = String::from_utf8(bytes)
				.map_err(|e| CommonError::FsRead(path, format!("UTF-8 decoding error: {}", e)))?;

			let (l, e) = app_state::lines_and_eol_from_text(&content);

			(
				l,
				e,
				encoding_detected,
				language_id_opt.unwrap_or_else(|| detect_language_id(Path::new(uri.path()))),
				false,
			)
		};

		let doc_state = DocumentState {
			uri:uri.clone(),

			language_id:actual_language_id,

			version:1,

			lines,

			eol,

			is_dirty:initial_dirty_state,

			encoding,
		};

		{
			let mut open_docs_guard = state.open_documents.lock().map_err(map_lock_error)?;

			info!("[Env DocumentProvider] Inserting new document {} into AppState.", uri);

			open_docs_guard.insert(uri.as_str().to_string(), doc_state.clone());
		}

		handlers::documents::notify_model_added(self.app_handle.clone(), &doc_state).await;

		info!("[Env DocumentProvider] Document {} opened (V1) and add notification sent.", uri);

		Ok(uri)
	}

	async fn save_document(&self, uri:Url) -> Result<bool, CommonError> {
		info!("[Env DocumentProvider] Saving document: {}", uri);

		if uri.scheme() != "file" {
			return Err(CommonError::NotImplemented(format!(
				"Saving non-file URI schemes: {}",
				uri.scheme()
			)));
		}

		let path = PathBuf::from(uri.path());

		let state = self.get_app_state();

		let (content_to_save, current_version) = {
			let open_docs_guard = state.open_documents.lock().map_err(map_lock_error)?;

			let doc_state = open_docs_guard
				.get(uri.as_str())
				.ok_or_else(|| CommonError::FsNotFound(path.clone()))?;

			if !doc_state.is_dirty {
				info!("[Env DocumentProvider] Document {} not dirty, no save needed.", uri);

				return Ok(true);
			}

			(doc_state.get_text(), doc_state.version)
		};

		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.require();

		fs_writer.write_file(&path, content_to_save.into_bytes(), true, true).await?;

		let mut was_dirty_before_save = true;

		{
			let mut open_docs_guard = state.open_documents.lock().map_err(map_lock_error)?;

			if let Some(doc_state_mut) = open_docs_guard.get_mut(uri.as_str()) {
				if doc_state_mut.version == current_version {
					doc_state_mut.is_dirty = false;
				} else {
					was_dirty_before_save = doc_state_mut.is_dirty;

					warn!(
						"[Env DocumentProvider] Document {} changed (v{} -> v{}) during save operation. Current dirty \
						 state: {}.",
						uri, current_version, doc_state_mut.version, doc_state_mut.is_dirty
					);
				}
			} else {
				warn!(
					"[Env DocumentProvider] Document {} was removed from state during save operation.",
					uri
				);

				was_dirty_before_save = false;
			}
		}

		handlers::documents::notify_model_saved(self.app_handle.clone(), &uri).await;

		if was_dirty_before_save {
			handlers::documents::notify_dirty_state_changed(self.app_handle.clone(), &uri, false).await;
		}

		info!(
			"[Env DocumentProvider] Document {} save process complete, notifications sent.",
			uri
		);

		Ok(true)
	}

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
				let ui_provider:Arc<dyn UiProvider + Send + Sync> = self.require();

				let save_opts_val = json!({ "defaultUri": { "scheme": original_uri.scheme(), "path": original_uri.path(), "external": original_uri.to_string(), "$mid": 1 }});

				let save_dialog_options:Option<SaveDialogOptions> = serde_json::from_value(save_opts_val)
					.map_err(|e| CommonError::InvalidArg("SaveDialogOptions".into(), e.to_string()))?;

				match ui_provider.show_save_dialog(save_dialog_options).await? {
					Some(path_buf) => {
						Url::from_file_path(path_buf).map_err(|_| {
							CommonError::InvalidArg(
								"new_uri_target".to_string(),
								"Selected path is not a valid URI".to_string(),
							)
						})?
					},

					None => {
						info!("[Env DocumentProvider] Save As cancelled by user for {}.", original_uri);

						return Ok(None);
					},
				}
			},
		};

		if new_uri.scheme() != "file" {
			return Err(CommonError::NotImplemented(format!(
				"Save As to non-file URI schemes: {}",
				new_uri.scheme()
			)));
		}

		let new_path = PathBuf::from(new_uri.path());

		let state = self.get_app_state();

		let (original_doc_content, original_lang_id, original_encoding, original_eol, original_is_untitled) = {
			let open_docs_guard = state.open_documents.lock().map_err(map_lock_error)?;

			let original_doc_state = open_docs_guard
				.get(original_uri.as_str())
				.ok_or_else(|| CommonError::FsNotFound(PathBuf::from(original_uri.path())))?;

			(
				original_doc_state.get_text(),
				original_doc_state.language_id.clone(),
				original_doc_state.encoding.clone(),
				original_doc_state.eol.clone(),
				original_uri.scheme() == "untitled",
			)
		};

		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.require();

		fs_writer
			.write_file(&new_path, original_doc_content.clone().into_bytes(), true, true)
			.await?;

		let mut open_docs_guard = state.open_documents.lock().map_err(map_lock_error)?;

		if original_is_untitled {
			if open_docs_guard.remove(original_uri.as_str()).is_some() {
				info!(
					"[Env DocumentProvider] Save As: Original untitled document {} removed from state.",
					original_uri
				);
			}
		}

		let (new_lines, _new_eol_after_save) = app_state::lines_and_eol_from_text(&original_doc_content);

		let new_doc_state = DocumentState {
			uri:new_uri.clone(),

			language_id:original_lang_id,

			version:1,

			lines:new_lines,

			eol:original_eol,

			is_dirty:false,

			encoding:original_encoding,
		};

		open_docs_guard.insert(new_uri.as_str().to_string(), new_doc_state.clone());

		drop(open_docs_guard);

		if original_is_untitled {
			handlers::documents::notify_model_removed(self.app_handle.clone(), &original_uri).await;
		}

		handlers::documents::notify_model_added(self.app_handle.clone(), &new_doc_state).await;

		handlers::documents::notify_model_saved(self.app_handle.clone(), &new_uri).await;

		info!(
			"[Env DocumentProvider] Document {} saved as {} and notifications sent.",
			original_uri, new_uri
		);

		Ok(Some(new_uri))
	}

	async fn apply_document_changes(
		&self,

		uri:Url,

		version_id:i64,

		changes_dto_val:Value,

		is_dirty:bool,

		is_undoing:bool,

		is_redoing:bool,
	) -> Result<(), CommonError> {
		info!(
			"[Env DocumentProvider] Applying {} changes for {} (Client V{}), dirty: {}, undo: {}, redo: {}",
			changes_dto_val.as_array().map_or(0, |a| a.len()),
			uri,
			version_id,
			is_dirty,
			is_undoing,
			is_redoing
		);

		let state = self.get_app_state();

		let mut open_docs_guard = state.open_documents.lock().map_err(map_lock_error)?;

		if let Some(doc_state) = open_docs_guard.get_mut(uri.as_str()) {
			let old_version = doc_state.version;

			if version_id <= old_version && changes_dto_val.as_array().map_or(false, |a| !a.is_empty()) {
				warn!(
					"[Env DocumentProvider] Stale content changes received for {}, V{} <= current V{}. Ignoring.",
					uri, version_id, old_version
				);

				return Ok(());
			}

			if version_id <= old_version && changes_dto_val.as_array().map_or(true, |a| a.is_empty()) {
				debug!(
					"[Env DocumentProvider] Stale or no-op version bump for {}, V{} <= current V{}. Ignoring.",
					uri, version_id, old_version
				);

				return Ok(());
			}

			trace!(
				"[Env DocumentProvider] Applying changes to DocumentState for {}. Old version: {}. New version: {}",
				uri, old_version, version_id
			);

			if let Err(e_str) = doc_state.apply_changes(version_id, &changes_dto_val) {
				error!(
					"[Env DocumentProvider] Error applying changes to DocumentState for {}: {}",
					uri, e_str
				);

				return Err(CommonError::Unknown(format!("Document change application failed: {}", e_str)));
			}

			doc_state.is_dirty = is_dirty;

			let updated_doc_eol_clone = doc_state.eol.clone();

			let updated_doc_version = doc_state.version;

			let updated_doc_is_dirty = doc_state.is_dirty;

			drop(open_docs_guard);

			handlers::documents::notify_model_changed(
				self.app_handle.clone(),
				&uri,
				updated_doc_version,
				&updated_doc_eol_clone,
				updated_doc_is_dirty,
				changes_dto_val,
				is_undoing,
				is_redoing,
			)
			.await;

			Ok(())
		} else {
			warn!(
				"[Env DocumentProvider] Document {} not found in AppState for apply_document_changes.",
				uri
			);

			Err(CommonError::FsNotFound(PathBuf::from(uri.path())))
		}
	}
}

impl Requires<Arc<dyn DocumentProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn DocumentProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl SecretsProvider for MountainEnvironment {
	async fn get_secret(&self, extension_id:String, key:String) -> Result<Option<String>, CommonError> {
		trace!("[Env SecretsProvider] Getting secret: ext_id='{}', key='{}'", extension_id, key);

		handlers::secrets::handle_get_secret(
			self.app_handle.clone(),
			json!({ "extensionId": extension_id, "key": key }),
		)
		.await
		.map(|val| val.as_str().map(String::from))
		.map_err(|e_str| CommonError::SecretsAccess(key, e_str))
	}

	async fn store_secret(&self, extension_id:String, key:String, value:String) -> Result<(), CommonError> {
		info!("[Env SecretsProvider] Storing secret: ext_id='{}', key='{}'", extension_id, key);

		handlers::secrets::handle_store_secret(
			self.app_handle.clone(),
			json!({ "extensionId": extension_id, "key": key, "value": value }),
		)
		.await
		.map(|_| ())
		.map_err(|e_str| CommonError::SecretsAccess(key, e_str))
	}

	async fn delete_secret(&self, extension_id:String, key:String) -> Result<(), CommonError> {
		info!(
			"[Env SecretsProvider] Deleting secret: ext_id='{}', key='{}'",
			extension_id, key
		);

		handlers::secrets::handle_delete_secret(
			self.app_handle.clone(),
			json!({ "extensionId": extension_id, "key": key }),
		)
		.await
		.map(|_| ())
		.map_err(|e_str| CommonError::SecretsAccess(key, e_str))
	}
}

impl Requires<Arc<dyn SecretsProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn SecretsProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl OutputChannelManager for MountainEnvironment {
	async fn register_channel(&self, name:String, language_id:Option<String>) -> Result<String, CommonError> {
		info!(
			"[Env OutputMgr] Registering channel: name='{}', lang_id='{:?}'",
			name, language_id
		);

		handlers::output::handle_register(self.app_handle.clone(), json!([name, Value::Null, language_id]))
			.await
			.map(|val| val.as_str().unwrap_or(&name).to_string())
			.map_err(|e_str| CommonError::OutputChannel(name, e_str))
	}

	async fn append(&self, channel_id:String, value:String) -> Result<(), CommonError> {
		trace!("[Env OutputMgr] Appending to channel: id='{}', len={}", channel_id, value.len());

		handlers::output::handle_append(self.app_handle.clone(), json!([channel_id, value]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::OutputChannel("append".to_string(), e_str))
	}

	async fn clear(&self, channel_id:String) -> Result<(), CommonError> {
		info!("[Env OutputMgr] Clearing channel: id='{}'", channel_id);

		handlers::output::handle_clear(self.app_handle.clone(), json!([channel_id]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::OutputChannel(channel_id, e_str))
	}

	async fn replace(&self, channel_id:String, value:String) -> Result<(), CommonError> {
		info!("[Env OutputMgr] Replacing content of channel: id='{}'", channel_id);

		handlers::output::handle_replace(self.app_handle.clone(), json!([channel_id, value]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::OutputChannel(channel_id, e_str))
	}

	async fn reveal(&self, channel_id:String, preserve_focus:bool) -> Result<(), CommonError> {
		info!(
			"[Env OutputMgr] Revealing channel: id='{}', preserve_focus={}",
			channel_id, preserve_focus
		);

		handlers::output::handle_reveal(self.app_handle.clone(), json!([channel_id, preserve_focus]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::OutputChannel(channel_id, e_str))
	}

	async fn close(&self, channel_id:String) -> Result<(), CommonError> {
		info!("[Env OutputMgr] Closing channel view: id='{}'", channel_id);

		handlers::output::handle_close(self.app_handle.clone(), json!([channel_id]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::OutputChannel(channel_id, e_str))
	}

	async fn dispose(&self, channel_id:String) -> Result<(), CommonError> {
		info!("[Env OutputMgr] Disposing channel: id='{}'", channel_id);

		handlers::output::handle_dispose(self.app_handle.clone(), json!([channel_id]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::OutputChannel(channel_id, e_str))
	}
}

impl Requires<Arc<dyn OutputChannelManager + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn OutputChannelManager + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl DiagnosticsManager for MountainEnvironment {
	async fn change_diagnostics(
		&self,

		owner:String,

		entries:Vec<(Url, Option<Vec<Value>>)>,
	) -> Result<(), CommonError> {
		info!(
			"[Env DiagMgr] Changing diagnostics: owner='{}', {} entries",
			owner,
			entries.len()
		);

		let entries_json:Vec<(Value, Option<Vec<Value>>)> = entries
			.into_iter()
			.map(|(url, markers_opt_val)| {
				let uri_components =
					json!({"scheme": url.scheme(), "path": url.path(), "external": url.to_string(), "$mid": 1});

				(uri_components, markers_opt_val)
			})
			.collect();

		handlers::diagnostics::handle_change_many(self.app_handle.clone(), json!([owner, entries_json]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::Diagnostics(e_str))
	}

	async fn clear_diagnostics_owner(&self, owner:String) -> Result<(), CommonError> {
		info!("[Env DiagMgr] Clearing diagnostics for owner: '{}'", owner);

		handlers::diagnostics::handle_clear(self.app_handle.clone(), json!([owner]))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::Diagnostics(e_str))
	}

	async fn get_all_diagnostics_for_uri(
		&self,

		uri_filter_opt:Option<Url>,
	) -> Result<Vec<(Url, Vec<Value>)>, CommonError> {
		trace!("[Env DiagMgr] Getting all diagnostics, filter: {:?}", uri_filter_opt);

		let uri_components_filter = uri_filter_opt
			.map(|url| json!({"scheme": url.scheme(), "path": url.path(), "external": url.to_string(), "$mid": 1}));

		let result_val = handlers::diagnostics::handle_get_diagnostics(
			self.app_handle.clone(),
			json!([uri_components_filter.unwrap_or(Value::Null)]),
		)
		.await
		.map_err(|e_str| CommonError::Diagnostics(e_str))?;

		let mut final_result = Vec::new();

		if let Some(arr) = result_val.as_array() {
			for tuple_val in arr {
				if let Some(tuple_arr) = tuple_val.as_array() {
					if let (Some(uri_comp_val), Some(markers_val_arr)) =
						(tuple_arr.get(0), tuple_arr.get(1).and_then(Value::as_array))
					{
						let uri_str = uri_comp_val.get("external").and_then(Value::as_str).ok_or_else(|| {
							CommonError::Diagnostics("Invalid URI component in get_all_diagnostics result".to_string())
						})?;

						let url = Url::parse(uri_str).map_err(|e| {
							CommonError::Diagnostics(format!(
								"Failed to parse URI '{}' in get_all_diagnostics result: {}",
								uri_str, e
							))
						})?;

						final_result.push((url, markers_val_arr.clone()));
					}
				}
			}
		}

		Ok(final_result)
	}
}

impl Requires<Arc<dyn DiagnosticsManager + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn DiagnosticsManager + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl CommandExecutor for MountainEnvironment {
	async fn execute_command(&self, command_id:String, args:Value) -> Result<Value, CommonError> {
		info!("[Env CommandExecutor] Executing command: '{}'", command_id);

		trace!("[Env CommandExecutor] Args: {:?}", args);

		let window = self
			.app_handle
			.get_window("main")
			.ok_or_else(|| CommonError::Unknown("Main window not found for command execution".to_string()))?;

		let runtime_state = self.app_handle.state::<Arc<AppRuntime>>();

		handlers::commands::handle_execute_command(
			self.app_handle.clone(),
			window,
			runtime_state.inner().clone(),
			json!({ "id": command_id, "args": args }),
		)
		.await
		.map_err(|e_str| CommonError::CommandExecution(command_id, e_str))
	}

	async fn register_command(&self, sidecar_id:String, command_id:String) -> Result<(), CommonError> {
		info!(
			"[Env CommandExecutor] Registering command: '{}' from sidecar '{}'",
			command_id, sidecar_id
		);

		handlers::commands::handle_register_command(self.app_handle.clone(), sidecar_id, json!({"id": command_id}))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::CommandRegistration(command_id, e_str))
	}

	async fn unregister_command(&self, sidecar_id:String, command_id:String) -> Result<(), CommonError> {
		info!(
			"[Env CommandExecutor] Unregistering command: '{}' from sidecar '{}'",
			command_id, sidecar_id
		);

		handlers::commands::handle_unregister_command(self.app_handle.clone(), sidecar_id, json!({"id": command_id}))
			.await
			.map(|_| ())
			.map_err(|e_str| CommonError::CommandRegistration(command_id, e_str))
	}

	async fn get_all_commands(&self) -> Result<Vec<String>, CommonError> {
		debug!("[Env CommandExecutor] Getting all commands");

		let runtime_state = self.app_handle.state::<Arc<AppRuntime>>();

		handlers::commands::handle_get_commands(self.app_handle.clone(), runtime_state.inner().clone())
			.await
			.and_then(|val| serde_json::from_value(val).map_err(|e| e.to_string()))
			.map_err(|e_str| CommonError::CommandList(e_str))
	}
}

impl Requires<Arc<dyn CommandExecutor + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn CommandExecutor + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl WorkspaceProvider for MountainEnvironment {
	async fn get_workspace_folders_info(&self) -> Result<Vec<(Url, String, usize)>, CommonError> {
		trace!("[Env WorkspaceProvider] Getting workspace folders info");

		let app_state = self.get_app_state();

		let folders_guard = app_state.workspace_folders.lock().map_err(map_lock_error)?;

		Ok(folders_guard.iter().map(|f| (f.uri.clone(), f.name.clone(), f.index)).collect())
	}

	async fn get_workspace_folder_info(&self, uri_to_match:Url) -> Result<Option<(Url, String, usize)>, CommonError> {
		debug!(
			"[Env WorkspaceProvider] Getting specific workspace folder info for: {}",
			uri_to_match
		);

		let app_state = self.get_app_state();

		let folders_guard = app_state.workspace_folders.lock().map_err(map_lock_error)?;

		Ok(folders_guard
			.iter()
			.find(|f| uri_to_match.as_str().starts_with(f.uri.as_str()))
			.map(|f| (f.uri.clone(), f.name.clone(), f.index)))
	}

	async fn get_workspace_name(&self) -> Result<Option<String>, CommonError> {
		debug!("[Env WorkspaceProvider] Getting workspace name");

		Ok(Some(self.get_app_state().get_workspace_name().map_err(CommonError::StateLock)?))
	}

	async fn get_workspace_configuration_path(&self) -> Result<Option<PathBuf>, CommonError> {
		debug!("[Env WorkspaceProvider] Getting workspace config path");

		Ok(self
			.get_app_state()
			.workspace_config_path
			.lock()
			.map_err(map_lock_error)?
			.clone())
	}

	async fn is_workspace_trusted(&self) -> Result<bool, CommonError> {
		debug!("[Env WorkspaceProvider] Getting workspace trust state");

		Ok(self.get_app_state().is_trusted.load(std::sync::atomic::Ordering::Relaxed))
	}

	async fn request_workspace_trust(&self, _options:Option<Value>) -> Result<bool, CommonError> {
		info!("[Env WorkspaceProvider] Requesting workspace trust");

		warn!("[Env WorkspaceProvider] requestWorkspaceTrust is STUBBED to return current trust state.");

		// TODO: Implement actual trust request flow (e.g., show dialog, update
		// AppState)
		Ok(self.get_app_state().is_trusted.load(std::sync::atomic::Ordering::Relaxed))
	}

	async fn find_files_in_workspace(
		&self,

		include:Value,

		exclude:Option<Value>,

		max_results:Option<usize>,

		use_ignore_files:bool,

		follow_symlinks:bool,
	) -> Result<Vec<Url>, CommonError> {
		info!(
			"[Env WorkspaceProvider] Finding files in workspace. Include='{:?}', Exclude='{:?}'",
			include, exclude
		);

		let params = json!([
			include,

			exclude.unwrap_or(Value::Null),

			{ "maxResults": max_results, "useIgnoreFiles": use_ignore_files, "followSymlinks": follow_symlinks }

		]);

		handlers::workspace::handle_find_files(self.app_handle.clone(), params)
			.await
			.and_then(|val_array| {
				val_array.as_array().map_or_else(
					|| Err(CommonError::Unknown("findFiles handler did not return an array".to_string())),
					|vec_val| {
						vec_val
							.iter()
							.map(|uri_comp_val| {
								let uri_str =
									uri_comp_val.get("external").and_then(Value::as_str).ok_or_else(|| {
										CommonError::Unknown("Invalid URI component in findFiles result".to_string())
									})?;

								Url::parse(uri_str).map_err(|e| {
									CommonError::Unknown(format!("Failed to parse URI from findFiles result: {}", e))
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

// Helper struct for UiProvider requests to Sky (payload for Tauri event)
#[derive(Serialize, Clone)]
struct UiRequestToSky<T:Serialize + Clone> {
	request_id:String,

	// Payload specific to the UI request type, matching what Sky expects
	payload:T,
}

#[async_trait]
impl UiProvider for MountainEnvironment {
	async fn show_message(
		&self,

		severity:MessageSeverity,

		message:String,

		options:Option<MessageOptions>,
	) -> Result<Option<String>, CommonError> {
		let severity_str = match severity {
			MessageSeverity::Info => "info",

			MessageSeverity::Warning => "warn",

			MessageSeverity::Error => "error",
		};

		info!(
			"[Env UiProvider] show_message: type='{}', msg='{}...', options: {:?}",
			severity_str,
			message.chars().take(50).collect::<String>(),
			options
		);

		let window = self
			.app_handle
			.get_window("main")
			.ok_or_else(|| CommonError::UiInteraction("Main window not found for show_message".to_string()))?;

		// If options contain items (buttons), it would need the sky:// event flow.
		// For simple messages without buttons/return value, we can use Tauri's blocking
		// dialog on a separate thread.
		if options
			.as_ref()
			.map_or(true, |o| o.items.is_empty() && !o.modal.unwrap_or(false))
		{
			let title = options
				.as_ref()
				.and_then(|o| o.title.as_ref())
				.map_or_else(|| format!("Land Editor - {}", severity_str.to_uppercase()), |t| t.clone());

			let msg_clone = message.clone();

			tokio::task::spawn_blocking(move || {
				tauri::api::dialog::message(Some(&window), title, msg_clone);
			})
			.await
			.map_err(|e| CommonError::UiInteraction(format!("Failed to spawn blocking task for dialog: {}", e)))?;

			Ok(None)
		} else {
			// Full flow for messages with buttons or modal messages that need a response
			let request_id = Uuid::new_v4().to_string();

			let (tx, rx) = oneshot::channel();

			{
				let app_state = self.get_app_state();

				let mut pending_guard = app_state.pending_ui_requests.lock().map_err(map_lock_error)?;

				pending_guard.insert(request_id.clone(), tx);
			}

			// Construct a payload Sky understands for showMessage
			let payload_data = json!({

				"severity": severity_str,

				"message": message,

				"options": options
			});

			let event_payload = UiRequestToSky { request_id:request_id.clone(), payload:payload_data };

			self.app_handle
				.emit_all("sky://ui/show-message-request", event_payload)
				.map_err(|e| CommonError::UiInteraction(format!("Failed to emit show_message request: {}", e)))?;

			let result_from_sky = match timeout(TokioDuration::from_secs(300), rx).await {
				// 5 min timeout
				Ok(Ok(Ok(value_from_sky))) => {
					// Assuming Sky sends back the selected item's string label or null if dismissed
					if value_from_sky.is_null() {
						Ok(None)
					} else if let Some(selected_item_str) = value_from_sky.as_str() {
						Ok(Some(selected_item_str.to_string()))
					} else {
						Err(CommonError::UiInteraction(
							"show_message response was not a string or null".to_string(),
						))
					}
				},

				Ok(Ok(Err(common_error_from_sky))) => Err(common_error_from_sky),

				Ok(Err(_channel_closed_err)) => {
					Err(CommonError::UiInteraction(format!(
						"show_message (ReqID: {}) response channel closed prematurely.",
						request_id
					)))
				},

				Err(_timeout_err) => {
					warn!("[Env UiProvider] show_message (ReqID: {}) timed out.", request_id);

					// Timeout means no selection or dialog dismissed
					Ok(None)
				},
			};

			self.get_app_state()
				.pending_ui_requests
				.lock()
				.map_err(map_lock_error)?
				.remove(&request_id);

			result_from_sky
		}
	}

	async fn show_open_dialog(&self, options:Option<OpenDialogOptions>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		let request_id = Uuid::new_v4().to_string();

		info!(
			"[Env UiProvider] show_open_dialog (ReqID: {}): options={:?}",
			request_id, options
		);

		let (tx, rx) = oneshot::channel();

		{
			let app_state = self.get_app_state();

			let mut pending_guard = app_state.pending_ui_requests.lock().map_err(map_lock_error)?;

			pending_guard.insert(request_id.clone(), tx);
		}

		let event_payload = UiRequestToSky { request_id:request_id.clone(), payload:options.clone() };

		self.app_handle
			.emit_all("sky://ui/show-open-dialog-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed to emit show_open_dialog request: {}", e)))?;

		let result_from_sky = match timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(value_from_sky))) => {
				if value_from_sky.is_null() {
					Ok(None)
				} else if let Some(paths_array) = value_from_sky.as_array() {
					let paths:Result<Vec<PathBuf>, _> =
						paths_array.iter().filter_map(|v| v.as_str().map(PathBuf::from)).collect();

					paths.map(Some).map_err(|_| {
						CommonError::UiInteraction("Invalid path string in open dialog response".to_string())
					})
				} else {
					Err(CommonError::UiInteraction(
						"Open dialog response was not an array of paths or null".to_string(),
					))
				}
			},

			Ok(Ok(Err(common_error_from_sky))) => Err(common_error_from_sky),

			Ok(Err(_channel_closed_err)) => {
				Err(CommonError::UiInteraction(format!(
					"Open dialog (ReqID: {}) response channel closed prematurely.",
					request_id
				)))
			},

			Err(_timeout_err) => {
				warn!("[Env UiProvider] show_open_dialog (ReqID: {}) timed out.", request_id);

				Ok(None)
			},
		};

		self.get_app_state()
			.pending_ui_requests
			.lock()
			.map_err(map_lock_error)?
			.remove(&request_id);

		result_from_sky
	}

	async fn show_save_dialog(&self, options:Option<SaveDialogOptions>) -> Result<Option<PathBuf>, CommonError> {
		let request_id = Uuid::new_v4().to_string();

		info!(
			"[Env UiProvider] show_save_dialog (ReqID: {}): options={:?}",
			request_id, options
		);

		let (tx, rx) = oneshot::channel();

		{
			let app_state = self.get_app_state();

			let mut pending_guard = app_state.pending_ui_requests.lock().map_err(map_lock_error)?;

			pending_guard.insert(request_id.clone(), tx);
		}

		let event_payload = UiRequestToSky { request_id:request_id.clone(), payload:options.clone() };

		self.app_handle
			.emit_all("sky://ui/show-save-dialog-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed to emit show_save_dialog request: {}", e)))?;

		let result_from_sky = match timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(value_from_sky))) => {
				if value_from_sky.is_null() {
					Ok(None)
				} else if let Some(path_str) = value_from_sky.as_str() {
					Ok(Some(PathBuf::from(path_str)))
				} else {
					Err(CommonError::UiInteraction(
						"Save dialog response was not a path string or null".to_string(),
					))
				}
			},

			Ok(Ok(Err(common_error_from_sky))) => Err(common_error_from_sky),

			Ok(Err(_channel_closed_err)) => {
				Err(CommonError::UiInteraction(format!(
					"Save dialog (ReqID: {}) response channel closed prematurely.",
					request_id
				)))
			},

			Err(_timeout_err) => {
				warn!("[Env UiProvider] show_save_dialog (ReqID: {}) timed out.", request_id);

				Ok(None)
			},
		};

		self.get_app_state()
			.pending_ui_requests
			.lock()
			.map_err(map_lock_error)?
			.remove(&request_id);

		result_from_sky
	}

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

		let (tx, rx) = oneshot::channel();

		{
			let app_state = self.get_app_state();

			let mut pending_guard = app_state.pending_ui_requests.lock().map_err(map_lock_error)?;

			pending_guard.insert(request_id.clone(), tx);
		}

		// QuickPickItem might contain non-serializable parts like `buttons`.
		// We need to serialize them carefully or define a DTO for Sky.
		let serializable_items = items
			.into_iter()
			.map(|item| {
				json!({

					"label": item.label,

					"description": item.description,

					"detail": item.detail,

					"picked": item.picked,

					"alwaysShow": item.always_show,

					// "buttons" field from QuickPickItem is not included here for simplicity.
					// If needed, they would require custom serialization logic or a DTO.
				})
			})
			.collect::<Vec<_>>();

		let payload_data = json!({ "items": serializable_items, "options": options });

		let event_payload = UiRequestToSky { request_id:request_id.clone(), payload:payload_data };

		self.app_handle
			.emit_all("sky://ui/show-quick-pick-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed to emit show_quick_pick request: {}", e)))?;

		let result_from_sky = match timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(value_from_sky))) => {
				if value_from_sky.is_null() {
					Ok(None)
				} else if options.as_ref().map_or(false, |o| o.can_pick_many) {
					if let Some(labels_array) = value_from_sky.as_array() {
						let labels:Result<Vec<String>, _> =
							labels_array.iter().filter_map(|v| v.as_str().map(String::from)).collect();

						labels.map(Some).map_err(|_| {
							CommonError::UiInteraction("Invalid string in quick pick multi-select response".to_string())
						})
					} else {
						Err(CommonError::UiInteraction(
							"Quick pick (multi) response was not an array of strings or null".to_string(),
						))
					}
				} else {
					if let Some(label_str) = value_from_sky.as_str() {
						Ok(Some(vec![label_str.to_string()]))
					} else {
						Err(CommonError::UiInteraction(
							"Quick pick (single) response was not a string or null".to_string(),
						))
					}
				}
			},

			Ok(Ok(Err(common_error_from_sky))) => Err(common_error_from_sky),

			Ok(Err(_channel_closed_err)) => {
				Err(CommonError::UiInteraction(format!(
					"Quick pick (ReqID: {}) response channel closed prematurely.",
					request_id
				)))
			},

			Err(_timeout_err) => {
				warn!("[Env UiProvider] show_quick_pick (ReqID: {}) timed out.", request_id);

				Ok(None)
			},
		};

		self.get_app_state()
			.pending_ui_requests
			.lock()
			.map_err(map_lock_error)?
			.remove(&request_id);

		result_from_sky
	}

	async fn show_input_box(&self, options:Option<InputBoxOptions>) -> Result<Option<String>, CommonError> {
		let request_id = Uuid::new_v4().to_string();

		info!("[Env UiProvider] show_input_box (ReqID: {}): options={:?}", request_id, options);

		let (tx, rx) = oneshot::channel();

		{
			let app_state = self.get_app_state();

			let mut pending_guard = app_state.pending_ui_requests.lock().map_err(map_lock_error)?;

			pending_guard.insert(request_id.clone(), tx);
		}

		let event_payload = UiRequestToSky { request_id:request_id.clone(), payload:options.clone() };

		self.app_handle
			.emit_all("sky://ui/show-input-box-request", event_payload)
			.map_err(|e| CommonError::UiInteraction(format!("Failed to emit show_input_box request: {}", e)))?;

		let result_from_sky = match timeout(TokioDuration::from_secs(300), rx).await {
			Ok(Ok(Ok(value_from_sky))) => {
				if value_from_sky.is_null() {
					Ok(None)
				} else if let Some(input_str) = value_from_sky.as_str() {
					Ok(Some(input_str.to_string()))
				} else {
					Err(CommonError::UiInteraction(
						"Input box response was not a string or null".to_string(),
					))
				}
			},

			Ok(Ok(Err(common_error_from_sky))) => Err(common_error_from_sky),

			Ok(Err(_channel_closed_err)) => {
				Err(CommonError::UiInteraction(format!(
					"Input box (ReqID: {}) response channel closed prematurely.",
					request_id
				)))
			},

			Err(_timeout_err) => {
				warn!("[Env UiProvider] show_input_box (ReqID: {}) timed out.", request_id);

				Ok(None)
			},
		};

		self.get_app_state()
			.pending_ui_requests
			.lock()
			.map_err(map_lock_error)?
			.remove(&request_id);

		result_from_sky
	}
}

impl Requires<Arc<dyn UiProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn UiProvider + Send + Sync> { Arc::new(self.clone()) }
}

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
			.map_err(|vine_err| {
				error!(
					"[Env IpcProvider] Vine error sending notification to {}: {}",
					sidecar_id, vine_err
				);

				CommonError::IpcError(vine_err.to_string())
			})
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
			.map_err(|vine_err| {
				error!("[Env IpcProvider] Vine error sending request to {}: {}", sidecar_id, vine_err);

				CommonError::IpcError(vine_err.to_string())
			})
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

		selector:Value,

		options:Option<Value>,
	) -> Result<u32, CommonError> {
		let app_state = self.get_app_state();

		let handle = app_state.get_next_provider_handle();

		let provider_type_appstate:app_state::LanguageProviderType = match provider_type_common {
			CommonProviderType::Hover => app_state::LanguageProviderType::Hover,

			CommonProviderType::Completion => app_state::LanguageProviderType::Completion,

			CommonProviderType::Definition => app_state::LanguageProviderType::Definition,

			CommonProviderType::Declaration => app_state::LanguageProviderType::Declaration,

			CommonProviderType::Implementation => app_state::LanguageProviderType::Implementation,

			CommonProviderType::TypeDefinition => app_state::LanguageProviderType::TypeDefinition,

			CommonProviderType::References => app_state::LanguageProviderType::References,

			CommonProviderType::DocumentHighlight => app_state::LanguageProviderType::DocumentHighlight,

			CommonProviderType::DocumentSymbol => app_state::LanguageProviderType::DocumentSymbol,

			CommonProviderType::WorkspaceSymbol => app_state::LanguageProviderType::WorkspaceSymbol,

			CommonProviderType::CodeAction => app_state::LanguageProviderType::CodeAction,

			CommonProviderType::CodeLens => app_state::LanguageProviderType::CodeLens,

			CommonProviderType::Formatting => app_state::LanguageProviderType::Formatting,

			CommonProviderType::RangeFormatting => app_state::LanguageProviderType::RangeFormatting,

			CommonProviderType::OnTypeFormatting => app_state::LanguageProviderType::OnTypeFormatting,

			CommonProviderType::Rename => app_state::LanguageProviderType::Rename,

			CommonProviderType::DocumentLink => app_state::LanguageProviderType::DocumentLink,

			CommonProviderType::Color => app_state::LanguageProviderType::Color,

			CommonProviderType::FoldingRange => app_state::LanguageProviderType::FoldingRange,

			CommonProviderType::SelectionRange => app_state::LanguageProviderType::SelectionRange,

			CommonProviderType::CallHierarchy => app_state::LanguageProviderType::CallHierarchy,

			CommonProviderType::TypeHierarchy => app_state::LanguageProviderType::TypeHierarchy,

			CommonProviderType::LinkedEditingRange => app_state::LanguageProviderType::LinkedEditingRange,

			CommonProviderType::InlayHints => app_state::LanguageProviderType::InlayHints,
		};

		info!(
			"[Env LangFeatRegistry] Registering {:?} (H:{}) from '{}'. Opts: {}",
			provider_type_appstate,
			handle,
			sidecar_id,
			options.is_some()
		);

		trace!("[Env LangFeatRegistry] Selector: {:?}, Options DTO: {:?}", selector, options);

		let registration = ProviderRegistration {
			handle,

			provider_type:provider_type_appstate,

			selector,

			sidecar_id,

			trigger_characters:options
				.as_ref()
				.and_then(|o| o.get("triggerCharacters"))
				.and_then(Value::as_array)
				.map(|arr| arr.iter().filter_map(Value::as_str).map(String::from).collect()),

			supports_resolve_details:options
				.as_ref()
				.and_then(|o| o.get("supportsResolveDetails"))
				.and_then(Value::as_bool),

			code_action_metadata:options.as_ref().and_then(|o| o.get("codeActionMetadata")).cloned(),

			signature_help_metadata:options.as_ref().and_then(|o| o.get("signatureHelpMetadata")).cloned(),
		};

		app_state
			.language_providers
			.lock()
			.map_err(map_lock_error)?
			.insert(handle, registration);

		Ok(handle)
	}

	async fn unregister_provider(&self, handle:u32) -> Result<(), CommonError> {
		info!("[Env LangFeatRegistry] Unregistering provider handle: {}", handle);

		if self
			.get_app_state()
			.language_providers
			.lock()
			.map_err(map_lock_error)?
			.remove(&handle)
			.is_none()
		{
			warn!(
				"[Env LangFeatRegistry] Attempted to unregister non-existent provider handle: {}",
				handle
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
			"[Env LangFeatRegistry] Querying providers for doc='{}', lang='{}', type='{:?}'",
			document_uri.as_str().split('/').last().unwrap_or_default(),
			language_id,
			provider_type_common
		);

		// Avoid using self.get_app_state() multiple times inside the lock
		let app_state_val = self.get_app_state();

		let providers_guard = app_state_val.language_providers.lock().map_err(map_lock_error)?;

		let target_provider_type_appstate:app_state::LanguageProviderType = match provider_type_common {
			CommonProviderType::Hover => app_state::LanguageProviderType::Hover,

			CommonProviderType::Completion => app_state::LanguageProviderType::Completion,

			CommonProviderType::Definition => app_state::LanguageProviderType::Definition,

			CommonProviderType::Declaration => app_state::LanguageProviderType::Declaration,

			CommonProviderType::Implementation => app_state::LanguageProviderType::Implementation,

			CommonProviderType::TypeDefinition => app_state::LanguageProviderType::TypeDefinition,

			CommonProviderType::References => app_state::LanguageProviderType::References,

			CommonProviderType::DocumentHighlight => app_state::LanguageProviderType::DocumentHighlight,

			CommonProviderType::DocumentSymbol => app_state::LanguageProviderType::DocumentSymbol,

			CommonProviderType::WorkspaceSymbol => app_state::LanguageProviderType::WorkspaceSymbol,

			CommonProviderType::CodeAction => app_state::LanguageProviderType::CodeAction,

			CommonProviderType::CodeLens => app_state::LanguageProviderType::CodeLens,

			CommonProviderType::Formatting => app_state::LanguageProviderType::Formatting,

			CommonProviderType::RangeFormatting => app_state::LanguageProviderType::RangeFormatting,

			CommonProviderType::OnTypeFormatting => app_state::LanguageProviderType::OnTypeFormatting,

			CommonProviderType::Rename => app_state::LanguageProviderType::Rename,

			CommonProviderType::DocumentLink => app_state::LanguageProviderType::DocumentLink,

			CommonProviderType::Color => app_state::LanguageProviderType::Color,

			CommonProviderType::FoldingRange => app_state::LanguageProviderType::FoldingRange,

			CommonProviderType::SelectionRange => app_state::LanguageProviderType::SelectionRange,

			CommonProviderType::CallHierarchy => app_state::LanguageProviderType::CallHierarchy,

			CommonProviderType::TypeHierarchy => app_state::LanguageProviderType::TypeHierarchy,

			CommonProviderType::LinkedEditingRange => app_state::LanguageProviderType::LinkedEditingRange,

			CommonProviderType::InlayHints => app_state::LanguageProviderType::InlayHints,
		};

		let mut matching_providers = Vec::new();

		for registration in providers_guard.values() {
			if registration.provider_type == target_provider_type_appstate {
				if handlers::config::match_document_selector(registration.selector, &document_uri, &language_id) {
					trace!(
						"[Env LangFeatRegistry] Match: Handle {}, Type {:?}, Doc {}, Lang {}",
						registration.handle,
						registration.provider_type,
						document_uri.as_str(),
						language_id
					);

					let mut options_map = serde_json::Map::new();

					if let Some(tc) = registration.trigger_characters {
						options_map.insert("triggerCharacters".to_string(), json!(tc));
					}

					if let Some(sr) = registration.supports_resolve_details {
						options_map.insert("supportsResolveDetails".to_string(), json!(sr));
					}

					if let Some(cam) = registration.code_action_metadata {
						options_map.insert("codeActionMetadata".to_string(), cam.clone());
					}

					if let Some(shm) = registration.signature_help_metadata {
						options_map.insert("signatureHelpMetadata".to_string(), shm.clone());
					}

					matching_providers.push(ProviderDescription {
						handle:registration.handle,

						sidecar_id:registration.sidecar_id.clone(),

						options:if options_map.is_empty() { None } else { Some(Value::Object(options_map)) },
					});
				}
			}
		}

		debug!(
			"[Env LangFeatRegistry] Found {} matching {:?} providers for doc='{}', lang='{}'",
			matching_providers.len(),
			provider_type_common,
			document_uri.as_str().split('/').last().unwrap_or_default(),
			language_id
		);

		Ok(matching_providers)
	}
}

impl Requires<Arc<dyn LanguageFeatureProviderRegistry + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn LanguageFeatureProviderRegistry + Send + Sync> { Arc::new(self.clone()) }
}
