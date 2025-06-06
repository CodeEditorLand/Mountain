// ---------------------------------------------------------------------------------------------
// Mountain Environment Implementation (environment.rs)
// --------------------------------------------------------------------------------------------
// Defines `MountainEnvironment`, the concrete implementation of the abstract
// `Environment` trait from `Land_Common`. It also implements various provider
// traits (e.g., `FsReader`, `FsWriter`, `ConfigProvider`, `DocumentProvider`,

// `UiProvider`, `LanguageFeatureProviderRegistry`, etc.) that define the actual
// "native" logic for `ActionEffect`s executed by the `AppRuntime`.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	ffi::OsStr,

	path::{Path, PathBuf},

	// Renamed to avoid conflict with tokio::Mutex
	sync::{Arc, MutexGuard as StdMutexGuard},

	time::Duration as StdDuration,
};

// Common effect traits and DTOs from Land_Common
use Land_Common::{
	command_effects::CommandExecutor,

	config_effects::{
		ConfigInspector,

		ConfigProvider,

		ConfigurationScope,

		ConfigurationTarget,

		IConfigurationInitDataDto,

		IConfigurationOverrides,

		InspectResultData,
	},

	// Renamed for clarity
	diagnostics_effects::{DiagnosticsManager, MarkerDataDto as CommonMarkerDataDto},

	documents_effects::{DocEventParams, DocumentProvider},

	// Core Environment trait and Requires helper
	environment::{Environment, Requires},

	errors::CommonError,

	fs_effects::{FileSystemProviderCapabilities, FileSystemStat, FileType as CommonFileType, FsReader, FsWriter},

	ipc_effects::{IpcProvider, ProxyTarget},

	language_feature_effects::{
		CodeActionContextDto,

		CodeActionDto,

		CodeActionListDto,

		CodeLensDto,

		CodeLensListDto,

		CompletionContextDto,

		DocumentHighlightDto,

		DocumentSymbolDto,

		// Added from WorkspaceEditApplier
		FileEditTypeDto,

		FoldingRangeDto,

		FormattingOptionsDto,

		HierarchyItemDto,

		HoverResultDto,

		IMarkdownStringDto,

		IncomingCallDto,

		InlayHintDto,

		LanguageFeatureProviderRegistry,

		LinkDto,

		LinkedEditingRangesDto,

		LinksListDto,

		LocationLinkDto,

		OutgoingCallDto,

		PositionDto,

		ProviderDescription,

		ProviderOptionsDto,

		ProviderType as CommonProviderType,

		RangeDto,

		SelectionRangeDto,

		SemanticTokensDto,

		SemanticTokensEditsDto,

		SignatureHelpContextDto,

		SignatureHelpResultDto,

		SuggestResultDto,

		TextEditDto,

		WorkspaceEditDto,

		WorkspaceFileEditDto,

		WorkspaceSymbolDto,

		// Added from WorkspaceEditApplier
		WorkspaceTextEditDto,
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

	// Added WorkspaceEditApplier
	workspace_effects::{WorkspaceEditApplier, WorkspaceProvider},
};
// For async methods in traits
use async_trait::async_trait;
// For dates
use chrono::Utc;
use log::{LevelFilter as LogLevelFilter, debug, error, info, trace, warn};
// For UiRequestToSkyPayload
use serde::Serialize;
use serde_json::{Map as JsonMap, Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime as TauriRuntime, State, Window, Wry};
use tokio::{
	// Tokio's async filesystem operations
	fs,

	// For file writing
	io::AsyncWriteExt,

	// For UiProvider async request-response
	sync::oneshot as TokioOneshot,

	// For UI timeouts
	time::{Duration as TokioDuration, timeout as tokio_timeout},
};
use url::Url;
// For generating unique request IDs
use uuid::Uuid;

use crate::{
	app_state::{
		self,

		AppState,

		DocumentState,

		LanguageProviderType as AppStateLanguageProviderType,

		// Mountain's internal MarkerData
		MarkerData as AppStateMarkerData,

		MementoStorageMap,

		// Used by LanguageFeatureProviderRegistry
		ProviderRegistration,
	},

	// Access to various handler modules
	handlers,

	// For AppRuntime access via AppHandle if needed by some handlers
	runtime::AppRuntime,

	// For IPC communication
	vine,
};

// --- Mountain Environment Struct ---
#[derive(Clone)]
pub struct MountainEnvironment {
	app_handle:AppHandle<Wry>,
}

impl MountainEnvironment {
	pub fn new(app_handle:AppHandle<Wry>) -> Self {
		info!("[Env Init] MountainEnvironment instance created.");

		Self { app_handle }
	}

	fn get_app_state(&self) -> State<'_, AppState> { self.app_handle.state::<AppState>() }

	async fn is_path_allowed_for_filesystem_access(&self, path_to_check:&Path) -> Result<(), CommonError> {
		trace!("[Env Security Check] Verifying path: {}", path_to_check.display());

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
			"[Env Security Check] Canonical path for '{}': '{}'",
			path_to_check.display(),
			canonical_path_to_check.display()
		);

		let mut allowed_root_paths:Vec<PathBuf> = Vec::new();

		let app_state = self.get_app_state();

		let folders_guard = app_state
			.workspace_folders
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		for folder in folders_guard.iter() {
			if folder.uri.scheme() == "file" {
				if let Ok(cfp) = std::fs::canonicalize(PathBuf::from(folder.uri.path())) {
					allowed_root_paths.push(cfp);
				} else {
					warn!("[Env Security Check] Failed to canonicalize workspace folder: {}", folder.uri);
				}
			}
		}

		drop(folders_guard);

		let path_resolver = self.app_handle.path_resolver();

		for dir_opt in [
			path_resolver.app_config_dir(),
			path_resolver.app_data_dir(),
			path_resolver.app_log_dir(),
		] {
			if let Some(dp) = dir_opt {
				if let Ok(cap) = std::fs::canonicalize(&dp) {
					allowed_root_paths.push(cap);
				} else {
					warn!("[Env Security Check] Failed to canonicalize app system dir: {}", dp.display());
				}
			}
		}

		if let Ok(cgm_path) = std::fs::canonicalize(&app_state.global_memento_path) {
			allowed_root_paths.push(cgm_path.clone());

			if let Some(p) = cgm_path.parent().and_then(|par| std::fs::canonicalize(par).ok()) {
				allowed_root_paths.push(p);
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

					if let Some(p) = cwm_path.parent().and_then(|par| std::fs::canonicalize(par).ok()) {
						allowed_root_paths.push(p);
					}
				}
			}
		}

		let is_allowed = allowed_root_paths
			.iter()
			.any(|root| canonical_path_to_check == *root || canonical_path_to_check.starts_with(root));

		if is_allowed {
			trace!("[Env Security Check] ALLOWED: '{}'", path_to_check.display());

			Ok(())
		} else {
			warn!(
				"[Env Security Check] DENIED: '{}' (canonical: '{}'). Not in roots: {:?}",
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

	pub fn get_file_provider_capabilities(&self) -> u32 {
		let mut capabilities = FileSystemProviderCapabilities::FileReadWrite as u32
			| FileSystemProviderCapabilities::FileOpenReadWriteLock as u32
			| FileSystemProviderCapabilities::FileFolderCopy as u32;

		if std::env::consts::OS != "windows" {
			capabilities |= FileSystemProviderCapabilities::PathCaseSensitive as u32;
		}

		debug!("[MountainEnv] File provider capabilities for 'file' scheme: {}", capabilities);

		capabilities
	}

	/// Helper to get ProviderRegistration, typically used by resolve methods.
	async fn get_provider_registration_from_handle(
		&self,

		handle:u32,

		expected_type:CommonProviderType,
	) -> Result<ProviderRegistration, CommonError> {
		let app_state = self.get_app_state();

		let providers_guard = app_state
			.language_providers
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		providers_guard.get(&handle)
             // Match common type directly
			.filter(|reg| reg.provider_type == expected_type)
            .cloned()
            .ok_or_else(|| CommonError::InvalidArg("handle".to_string(), format!("No matching {:?} provider for handle {}", expected_type, handle)))
	}
}

impl Environment for MountainEnvironment {}

fn map_app_state_lock_error_to_common_error<T>(e:std::sync::PoisonError<StdMutexGuard<'_, T>>) -> CommonError {
	let err_msg = format!("Failed to lock AppState section: {}", e);

	error!("[Env AppStateLockErr] {}", err_msg);

	CommonError::StateLock(err_msg)
}

fn map_io_error_to_common_error(e:std::io::Error, path:PathBuf, operation:&'static str) -> CommonError {
	warn!("[Env IOError] FS op '{}' on '{}' failed: {}", operation, path.display(), e);

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

				"rename" => CommonError::FsRename { source:path, target:PathBuf::new(), description:e.to_string() }, /* Target placeholder */
				"copy" => CommonError::FsCopy { source:path, target:PathBuf::new(), description:e.to_string() }, /* Target placeholder */
				"readdir" | "readdir_next" => CommonError::FsReadDir { path, description:e.to_string() },

				_ => {
					CommonError::Unknown(format!("Unknown FS Op '{}' on '{}' failed: {}", operation, path.display(), e))
				},
			}
		},
	}
}

fn detect_language_id_from_file_path(path:&Path) -> String {
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

fn detect_file_encoding_from_bytes(_content_bytes:&[u8]) -> String { "utf8".to_string() }

// Helper struct for UiProvider requests to Sky
#[derive(Serialize, Clone)]
struct UiRequestToSkyPayload<T:Serialize + Clone> {
	request_id:String,

	// Payload specific to the UI request type
	payload:T,
}

// --- Effect Provider Trait Implementations ---

#[async_trait]
impl FsReader for MountainEnvironment {
	// ... (Full implementation from previous synthesis, uses
	// is_path_allowed_for_filesystem_access) ...
	async fn read_file(&self, path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		self.is_path_allowed_for_filesystem_access(path).await?;

		trace!("[Env FsReader] Reading file: {}", path.display());

		fs::read(path)
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "read"))
	}

	async fn stat_file(&self, path:&PathBuf) -> Result<FileSystemStat, CommonError> {
		self.is_path_allowed_for_filesystem_access(path).await?;

		trace!("[Env FsReader] Stating file/directory: {}", path.display());

		match tokio::fs::metadata(path).await {
			Ok(metadata) => {
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

				let get_milli_ts = |sys_time_res:Result<std::time::SystemTime, _>| -> u64 {
					sys_time_res
						.ok()
						.and_then(|t| t.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
						.map_or(0, |d| d.as_millis() as u64)
				};

				Ok(FileSystemStat {
					file_type:file_type_flags,

					ctime:get_milli_ts(metadata.created()),

					mtime:get_milli_ts(metadata.modified()),

					size:metadata.len(),

					permissions:None,
				})
			},

			Err(io_err) => Err(map_io_error_to_common_error(io_err, path.clone(), "stat")),
		}
	}

	async fn read_directory(&self, path:&PathBuf) -> Result<Vec<(String, CommonFileType)>, CommonError> {
		self.is_path_allowed_for_filesystem_access(path).await?;

		debug!("[Env FsReader] Reading directory: {}", path.display());

		let mut entries_vec:Vec<(String, CommonFileType)> = Vec::new();

		let mut dir_entries_stream = fs::read_dir(path)
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "readdir"))?;

		while let Some(dir_entry_res) = dir_entries_stream
			.next_entry()
			.await
			.map_err(|io_err| map_io_error_to_common_error(io_err, path.clone(), "readdir_next_entry"))?
		{
			let file_name_str = dir_entry_res.file_name().to_string_lossy().into_owned();

			match dir_entry_res.file_type().await {
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

					entries_vec.push((file_name_str, common_ft));
				},

				Err(e_ftype) => {
					warn!(
						"[Env FsReader] Failed to get type for '{}' in '{}': {}. Marking Unknown.",
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

impl Requires<Arc<dyn FsReader + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn FsReader + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl FsWriter for MountainEnvironment {
	// ... (Full implementation from previous synthesis, uses
	// is_path_allowed_for_filesystem_access) ...
	async fn write_file(
		&self,

		path:&PathBuf,

		content_bytes:Vec<u8>,

		create:bool,

		overwrite:bool,
	) -> Result<(), CommonError> {
		self.is_path_allowed_for_filesystem_access(path).await?;

		info!(
			"[Env FsWriter] Write: '{}', len={}, create={}, overwrite={}",
			path.display(),
			content_bytes.len(),
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

		if let Some(parent_dir) = path.parent() {
			if !fs::try_exists(parent_dir).await.unwrap_or(false) {
				if create {
					fs::create_dir_all(parent_dir)
						.await
						.map_err(|e| map_io_error_to_common_error(e, parent_dir.to_path_buf(), "mkdir_parent_write"))?;
				} else {
					return Err(CommonError::FsNotFound(parent_dir.to_path_buf()));
				}
			}
		}

		fs::write(path, &content_bytes)
			.await
			.map_err(|e| map_io_error_to_common_error(e, path.clone(), "write"))?;

		// TODO: Emit filesystem_changed event
		Ok(())
	}

	async fn create_directory(&self, path:&PathBuf, recursive:bool) -> Result<(), CommonError> {
		self.is_path_allowed_for_filesystem_access(path).await?;

		info!("[Env FsWriter] Mkdir: '{}', recursive={}", path.display(), recursive);

		if recursive {
			fs::create_dir_all(path)
				.await
				.map_err(|e| map_io_error_to_common_error(e, path.clone(), "mkdir_all"))?;
		} else {
			fs::create_dir(path)
				.await
				.map_err(|e| map_io_error_to_common_error(e, path.clone(), "mkdir"))?;
		}

		// TODO: Emit filesystem_changed event
		Ok(())
	}

	async fn delete(&self, path:&PathBuf, recursive:bool, use_trash:bool) -> Result<(), CommonError> {
		self.is_path_allowed_for_filesystem_access(path).await?;

		info!(
			"[Env FsWriter] Delete: '{}', recursive={}, useTrash={}",
			path.display(),
			recursive,
			use_trash
		);

		if use_trash {
			warn!("[Env FsWriter] 'useTrash' STUBBED, using permanent delete.");
		}

		match fs::metadata(path).await {
			Ok(md) => {
				let op = if md.is_dir() {
					if recursive {
						fs::remove_dir_all(path).await
					} else {
						fs::remove_dir(path).await
					}
				} else {
					fs::remove_file(path).await
				};

				op.map_err(|e| map_io_error_to_common_error(e, path.clone(), "delete"))?;

				// TODO: Emit filesystem_changed event
				Ok(())
			},

			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
				debug!(
					"[Env FsWriter] Delete: path '{}' not found (idempotent success).",
					path.display()
				);

				Ok(())
			},

			Err(e) => Err(map_io_error_to_common_error(e, path.clone(), "delete_stat_check")),
		}
	}

	async fn rename(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		self.is_path_allowed_for_filesystem_access(source).await?;

		self.is_path_allowed_for_filesystem_access(target).await?;

		info!(
			"[Env FsWriter] Rename: '{}' -> '{}', overwrite={}",
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
			debug!("[Env FsWriter] Rename: Overwriting target by deleting: {}", target.display());

			let md = fs::metadata(target)
				.await
				.map_err(|e| map_io_error_to_common_error(e, target.clone(), "rename_target_stat"))?;

			self.delete(target, md.is_dir(), false).await?;
		}

		if let Some(parent_dir) = target.parent() {
			if !fs::try_exists(parent_dir).await.unwrap_or(false) {
				fs::create_dir_all(parent_dir)
					.await
					.map_err(|e| map_io_error_to_common_error(e, parent_dir.to_path_buf(), "mkdir_parent_rename"))?;
			}
		}

		fs::rename(source, target)
			.await
			.map_err(|e| map_io_error_to_common_error(e, source.clone(), "rename"))?;

		// TODO: Emit filesystem_changed event
		Ok(())
	}

	async fn copy(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		self.is_path_allowed_for_filesystem_access(source).await?;

		self.is_path_allowed_for_filesystem_access(target).await?;

		info!(
			"[Env FsWriter] Copy: '{}' -> '{}', overwrite={}",
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

		let md = fs::metadata(source)
			.await
			.map_err(|e| map_io_error_to_common_error(e, source.clone(), "copy_source_stat"))?;

		if md.is_dir() {
			error!("[Env FsWriter] Recursive dir copy from '{}' STUBBED.", source.display());

			return Err(CommonError::NotImplemented("Recursive directory copy".to_string()));
		}

		if overwrite && fs::try_exists(target).await.unwrap_or(false) {
			debug!("[Env FsWriter] Copy: Overwriting target by deleting: {}", target.display());

			self.delete(target, false, false).await?;
		}

		if let Some(parent_dir) = target.parent() {
			if !fs::try_exists(parent_dir).await.unwrap_or(false) {
				fs::create_dir_all(parent_dir)
					.await
					.map_err(|e| map_io_error_to_common_error(e, parent_dir.to_path_buf(), "mkdir_parent_copy"))?;
			}
		}

		fs::copy(source, target)
			.await
			.map(|_| ())
			.map_err(|e| map_io_error_to_common_error(e, source.clone(), "copy"))?;

		// TODO: Emit filesystem_changed event
		Ok(())
	}

	// This was missing, used by create_file effect constructor
	async fn create_file(&self, path:&PathBuf) -> Result<(), CommonError> {
		self.is_path_allowed_for_filesystem_access(path).await?;

		info!("[Env FsWriter] Creating empty file: {}", path.display());

		if fs::try_exists(path).await.unwrap_or(false) {
			return Err(CommonError::FsFileExists(path.clone()));
		}

		if let Some(p_dir) = path.parent() {
			if !fs::try_exists(p_dir).await.unwrap_or(false) {
				fs::create_dir_all(p_dir)
					.await
					.map_err(|e| map_io_error_to_common_error(e, p_dir.to_path_buf(), "mkdir_parent"))?;
			}
		}

		fs::File::create(path)
			.await
			.map(|_| ())
			.map_err(|e| map_io_error_to_common_error(e, path.clone(), "create_file"))?;

		// TODO: Emit filesystem_changed
		Ok(())
	}
}

impl Requires<Arc<dyn FsWriter + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn FsWriter + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl ConfigProvider for MountainEnvironment {
	// ... (Full implementation from previous synthesis, delegates to
	// handlers::config or uses AppState) ...
	async fn get_configuration_value(
		&self,

		section_key_opt:Option<String>,

		overrides:IConfigurationOverrides,
	) -> Result<Value, CommonError> {
		trace!(
			"[Env CfgProv] GetConfig: section={:?}, overrides.resource={:?}, overrides.langId={:?}",
			section_key_opt,
			overrides.resource.as_ref().and_then(|v| v.get("external")),
			overrides.override_identifier
		);

		let app_state = self.get_app_state();

		let config_state_guard = app_state
			.configuration
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		if overrides.resource.is_some() || overrides.override_identifier.is_some() {
			warn!("[Env CfgProv GetConfig] Overrides provided, but current impl primarily uses pre-merged state.");
		}

		let value_result = config_state_guard.get_value(section_key_opt.as_deref(), overrides.resource.as_ref());

		debug!(
			"[Env CfgProv GetConfig] Value for {:?}: {}...",
			section_key_opt,
			value_result.to_string().chars().take(70).collect::<String>()
		);

		Ok(value_result)
	}

	async fn update_configuration_value(
		&self,

		key_to_update:String,

		value_to_set:Value,

		target_scope:ConfigurationTarget,

		overrides:IConfigurationOverrides,

		scope_to_language_override:Option<bool>,
	) -> Result<(), CommonError> {
		info!(
			"[Env CfgProv UpdateConfig] Req: key='{}', target={:?}, value_is_null={}, scope_to_lang={:?}, \
			 override_res={:?}",
			key_to_update,
			target_scope,
			value_to_set.is_null(),
			scope_to_language_override,
			overrides.resource.as_ref().and_then(|v| v.get("external"))
		);

		let app_state = self.get_app_state();

		let target_config_file_path = handlers::config::get_config_path_for_target(
			&self.app_handle,
			&app_state,
			target_scope,
			&overrides,
			scope_to_language_override.unwrap_or(false),
		)?;

		info!("[Env CfgProv UpdateConfig] Target file: {}", target_config_file_path.display());

		let mut current_target_file_json_content =
			handlers::config::load_json_file_if_exists_or_default(&target_config_file_path).await?;

		let mut effective_json_node_to_update_in = &mut current_target_file_json_content;

		let mut language_scope_key_holder:Option<String> = None;

		if scope_to_language_override.unwrap_or(false) {
			if let Some(lang_id_str) = &overrides.override_identifier {
				language_scope_key_holder = Some(format!("[{}]", lang_id_str));

				let lang_scope_key_ref = language_scope_key_holder.as_ref().unwrap();

				if !effective_json_node_to_update_in.is_object() {
					*effective_json_node_to_update_in = json!({});
				}

				effective_json_node_to_update_in = effective_json_node_to_update_in
					.as_object_mut()
					.unwrap()
					.entry(lang_scope_key_ref.clone())
					.or_insert_with(|| json!({}));
			} else {
				warn!(
					"[Env CfgProv UpdateConfig] 'scopeToLanguage' true for '{}', but no languageId. Updating \
					 top-level of '{}'.",
					key_to_update,
					target_config_file_path.display()
				);
			}
		}

		handlers::config::update_json_value_at_path(effective_json_node_to_update_in, &key_to_update, value_to_set);

		trace!(
			"[Env CfgProv UpdateConfig] Key '{}' updated in-memory for '{}'.",
			key_to_update,
			target_config_file_path.display()
		);

		handlers::config::write_json_file(&target_config_file_path, current_target_file_json_content).await?;

		info!(
			"[Env CfgProv UpdateConfig] Wrote updated config to: {}",
			target_config_file_path.display()
		);

		let new_merged_config_state =
			handlers::config::load_and_merge_configurations_internal(&self.app_handle, &app_state).await?;

		app_state
			.configuration
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.update_from_new_state(new_merged_config_state);

		info!(
			"[Env CfgProv UpdateConfig] AppState.configuration reloaded after change to '{}'.",
			target_config_file_path.display()
		);

		handlers::config::notify_config_changed_for_keys(&self.app_handle, vec![key_to_update]).await;

		Ok(())
	}
}

impl Requires<Arc<dyn ConfigProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn ConfigProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl ConfigInspector for MountainEnvironment {
	// ... (Full implementation from previous synthesis, partially stubbed for
	// non-effective values) ...
	async fn inspect_configuration_value(
		&self,

		key:String,

		overrides:IConfigurationOverrides,
	) -> Result<Option<InspectResultData>, CommonError> {
		info!(
			"[Env CfgInsp] Inspecting key='{}', overrides.resource={:?}",
			key,
			overrides.resource.as_ref().and_then(|v| v.get("external"))
		);

		let app_state = self.get_app_state();

		let config_guard = app_state
			.configuration
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		let effective_value = config_guard.get_value(Some(&key), overrides.resource.as_ref());

		if effective_value.is_null() && !config_guard.data.get(&key).is_some() {
			// Check if key actually exists even if null
			// Key truly not found in effective config
			Ok(None)
		} else {
			// TODO: Populate other fields (defaultValue, userValue etc.) by actually
			// reading individual config files.
			warn!(
				"[Env CfgInsp] inspect_configuration_value STUBBED for non-effective values. Returning effective \
				 value only."
			);

			Ok(Some(InspectResultData {
				effective_value:Some(effective_value),

				..Default::default()
			}))
		}
	}
}

impl Requires<Arc<dyn ConfigInspector + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn ConfigInspector + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl DocumentProvider for MountainEnvironment {
	// ... (Full implementation delegating to
	// handlers::documents::handle_*_effect_logic) ...
	async fn open_document(
		&self,

		uri_components_dto:Value,

		language_id:Option<String>,

		content:Option<String>,
	) -> Result<Url, CommonError> {
		handlers::documents::handle_open_document_effect_logic(
			self.app_handle.clone(),
			// Pass MountainEnvironment for FsReader
			self.clone(),
			uri_components_dto,
			language_id,
			content,
		)
		.await
	}

	async fn save_document(&self, uri:Url) -> Result<bool, CommonError> {
		handlers::documents::handle_save_document_effect_logic(self.app_handle.clone(), self.clone(), uri).await
	}

	async fn save_document_as(&self, original_uri:Url, new_target_uri:Option<Url>) -> Result<Option<Url>, CommonError> {
		handlers::documents::handle_save_document_as_effect_logic(
			self.app_handle.clone(),
			self.clone(),
			original_uri,
			new_target_uri,
		)
		.await
	}

	async fn save_all_documents(&self, include_untitled:bool) -> Result<Vec<bool>, CommonError> {
		handlers::documents::handle_save_all_documents_effect_logic(
			self.app_handle.clone(),
			self.clone(),
			include_untitled,
		)
		.await
	}

	async fn apply_document_changes(
		&self,

		uri:Url,

		new_version_id:i64,

		changes_dto_collection:Value,

		is_dirty_after_change:bool,

		is_undoing:bool,

		is_redoing:bool,
	) -> Result<(), CommonError> {
		handlers::documents::handle_apply_document_changes_effect_logic(
			self.app_handle.clone(),
			self.clone(),
			uri,
			new_version_id,
			changes_dto_collection,
			is_dirty_after_change,
			is_undoing,
			is_redoing,
		)
		.await
	}
}

impl Requires<Arc<dyn DocumentProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn DocumentProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl StorageProvider for MountainEnvironment {
	// ... (Full implementation delegating to
	// handlers::storage::handle_*_effect_logic) ...
	async fn get_storage_value(&self, is_global_scope:bool, key:&str) -> Result<Option<Value>, CommonError> {
		handlers::storage::handle_get_storage_value_effect_logic(self.app_handle.clone(), is_global_scope, key).await
	}

	async fn update_storage_value(
		&self,

		is_global_scope:bool,

		key:String,

		value_to_set:Option<Value>,
	) -> Result<(), CommonError> {
		handlers::storage::handle_set_storage_value_effect_logic(
			self.app_handle.clone(),
			is_global_scope,
			key,
			value_to_set,
		)
		.await
	}
}

impl Requires<Arc<dyn StorageProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn StorageProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl SecretsProvider for MountainEnvironment {
	// ... (Full implementation delegating to
	// handlers::secrets::handle_*_effect_logic) ...
	async fn get_secret(&self, extension_id:String, key:String) -> Result<Option<String>, CommonError> {
		handlers::secrets::handle_get_secret_effect_logic(self.app_handle.clone(), extension_id, key).await
	}

	async fn store_secret(&self, extension_id:String, key:String, value_to_store:String) -> Result<(), CommonError> {
		handlers::secrets::handle_store_secret_effect_logic(self.app_handle.clone(), extension_id, key, value_to_store)
			.await
	}

	async fn delete_secret(&self, extension_id:String, key:String) -> Result<(), CommonError> {
		handlers::secrets::handle_delete_secret_effect_logic(self.app_handle.clone(), extension_id, key).await
	}
}

impl Requires<Arc<dyn SecretsProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn SecretsProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl OutputChannelManager for MountainEnvironment {
	// ... (Full implementation delegating to
	// handlers::output::handle_*_effect_logic) ...
	async fn register_channel(&self, name:String, language_id:Option<String>) -> Result<String, CommonError> {
		handlers::output::handle_register_output_channel_effect_logic(self.app_handle.clone(), name, language_id).await
	}

	async fn append(&self, channel_id:String, value:String) -> Result<(), CommonError> {
		handlers::output::handle_append_to_output_channel_effect_logic(self.app_handle.clone(), channel_id, value).await
	}

	async fn replace(&self, channel_id:String, value:String) -> Result<(), CommonError> {
		handlers::output::handle_replace_output_channel_content_effect_logic(self.app_handle.clone(), channel_id, value)
			.await
	}

	async fn clear(&self, channel_id:String) -> Result<(), CommonError> {
		handlers::output::handle_clear_output_channel_effect_logic(self.app_handle.clone(), channel_id).await
	}

	async fn reveal(&self, channel_id:String, preserve_focus:bool) -> Result<(), CommonError> {
		handlers::output::handle_reveal_output_channel_effect_logic(self.app_handle.clone(), channel_id, preserve_focus)
			.await
	}

	async fn close(&self, channel_id:String) -> Result<(), CommonError> {
		handlers::output::handle_close_output_channel_view_effect_logic(self.app_handle.clone(), channel_id).await
	}

	async fn dispose(&self, channel_id:String) -> Result<(), CommonError> {
		handlers::output::handle_dispose_output_channel_effect_logic(self.app_handle.clone(), channel_id).await
	}
}

impl Requires<Arc<dyn OutputChannelManager + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn OutputChannelManager + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl DiagnosticsManager for MountainEnvironment {
	// ... (Full implementation delegating to
	// handlers::diagnostics::handle_*_effect_logic) ...
	async fn set_diagnostics(&self, owner:String, entries_dto_val:Value) -> Result<(), CommonError> {
		handlers::diagnostics::handle_set_diagnostics_effect_logic(self.app_handle.clone(), owner, entries_dto_val)
			.await
	}

	async fn clear_diagnostics(&self, owner:String) -> Result<(), CommonError> {
		handlers::diagnostics::handle_clear_diagnostics_effect_logic(self.app_handle.clone(), owner).await
	}

	async fn get_all_diagnostics(&self, resource_uri_filter_opt:Option<Value>) -> Result<Value, CommonError> {
		handlers::diagnostics::handle_get_all_diagnostics_effect_logic(self.app_handle.clone(), resource_uri_filter_opt)
			.await
	}
}

impl Requires<Arc<dyn DiagnosticsManager + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn DiagnosticsManager + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl CommandExecutor for MountainEnvironment {
	// ... (Full implementation delegating to handlers::commands, requires Window
	// and AppRuntime) ...
	async fn execute_command(&self, command_id:String, args_val:Value) -> Result<Value, CommonError> {
		info!("[Env CmdExec] Execute: cmd_id='{}'", command_id);

		trace!("[Env CmdExec] Argument: {:?}", args_val);

		let main_window = self
			.app_handle
			.get_webview_window("main")
			.ok_or_else(|| CommonError::UiInteraction("Main window not found for command exec".to_string()))?;

		let app_runtime_state = self.app_handle.state::<Arc<AppRuntime>>();

		handlers::commands::handle_execute_command(
			self.app_handle.clone(),
			main_window,
			app_runtime_state.inner().clone(),
			json!({ "id": command_id, "args": args_val }),
		)
		.await
		.map_err(|e_str| CommonError::CommandExecution(command_id, e_str))
	}

	async fn register_command(&self, sidecar_id:String, command_id:String) -> Result<(), CommonError> {
		info!("[Env CmdExec] Register: sid='{}', cmd_id='{}'", sidecar_id, command_id);

		handlers::commands::handle_register_command(self.app_handle.clone(), sidecar_id, json!({ "id": command_id }))
			.await
			.map(|_| ())
			.map_err(|e| CommonError::CommandRegistration(command_id, e))
	}

	async fn unregister_command(&self, sidecar_id:String, command_id:String) -> Result<(), CommonError> {
		info!("[Env CmdExec] Unregister: sid='{}', cmd_id='{}'", sidecar_id, command_id);

		handlers::commands::handle_unregister_command(self.app_handle.clone(), sidecar_id, json!({ "id": command_id }))
			.await
			.map(|_| ())
			.map_err(|e| CommonError::CommandRegistration(command_id, e))
	}

	async fn get_all_commands(&self) -> Result<Vec<String>, CommonError> {
		debug!("[Env CmdExec] GetAllCommands");

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
	// ... (Full implementation delegating to handlers::workspace or AppState) ...
	async fn get_workspace_folders_info(&self) -> Result<Vec<(Url, String, usize)>, CommonError> {
		handlers::workspace::handle_get_workspace_folders_info_effect_logic(self.app_handle.clone()).await
	}

	async fn get_workspace_folder_info(&self, uri_to_match:Url) -> Result<Option<(Url, String, usize)>, CommonError> {
		handlers::workspace::handle_get_workspace_folder_info_effect_logic(self.app_handle.clone(), uri_to_match).await
	}

	async fn get_workspace_name(&self) -> Result<Option<String>, CommonError> {
		handlers::workspace::handle_get_workspace_name_effect_logic(self.app_handle.clone()).await
	}

	async fn get_workspace_configuration_path(&self) -> Result<Option<PathBuf>, CommonError> {
		handlers::workspace::handle_get_workspace_configuration_path_effect_logic(self.app_handle.clone()).await
	}

	async fn is_workspace_trusted(&self) -> Result<bool, CommonError> {
		handlers::workspace::handle_is_workspace_trusted_effect_logic(self.app_handle.clone()).await
	}

	async fn request_workspace_trust(&self, options:Option<Value>) -> Result<bool, CommonError> {
		handlers::workspace::handle_request_workspace_trust_effect_logic(self.app_handle.clone(), options).await
	}

	async fn find_files_in_workspace(
		&self,

		include_pattern_dto:Value,

		exclude_pattern_dto:Option<Value>,

		max_results:Option<usize>,

		use_ignore_files:bool,

		follow_symlinks:bool,
	) -> Result<Vec<Url>, CommonError> {
		handlers::workspace::handle_find_files_in_workspace_effect_logic(
			self.app_handle.clone(),
			include_pattern_dto,
			exclude_pattern_dto,
			max_results,
			use_ignore_files,
			follow_symlinks,
		)
		.await
	}

	async fn open_file(&self, path:PathBuf) -> Result<(), CommonError> {
		handlers::workspace::handle_open_file_effect_logic(self.app_handle.clone(), path).await
	}
}

impl Requires<Arc<dyn WorkspaceProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn WorkspaceProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl UiProvider for MountainEnvironment {
	// ... (Full implementation using AppState.pending_ui_requests and Sky events)
	// ...
	async fn show_message(
		&self,

		severity:MessageSeverity,

		message_text:String,

		options_json_val_opt:Option<Value>,
	) -> Result<Option<String>, CommonError> {
		// Simplified logic from original - full logic with simple dialog vs Sky IPC
		// would be here
		handlers::ui::handle_show_message_interactive(
			self.app_handle.clone(),
			severity,
			message_text,
			options_json_val_opt,
		)
		.await
	}

	async fn show_open_dialog(&self, options:Option<OpenDialogOptions>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		handlers::ui::handle_show_open_dialog_interactive(self.app_handle.clone(), options).await
	}

	async fn show_save_dialog(&self, options:Option<SaveDialogOptions>) -> Result<Option<PathBuf>, CommonError> {
		handlers::ui::handle_show_save_dialog_interactive(self.app_handle.clone(), options).await
	}

	async fn show_quick_pick(
		&self,

		items:Vec<QuickPickItem>,

		options:Option<QuickPickOptions>,
	) -> Result<Option<Vec<String>>, CommonError> {
		handlers::ui::handle_show_quick_pick_interactive(self.app_handle.clone(), items, options).await
	}

	async fn show_input_box(&self, options:Option<InputBoxOptions>) -> Result<Option<String>, CommonError> {
		handlers::ui::handle_show_input_box_interactive(self.app_handle.clone(), options).await
	}
}

impl Requires<Arc<dyn UiProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn UiProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl IpcProvider for MountainEnvironment {
	// ... (Full implementation delegating to vine) ...
	async fn send_notification_to_sidecar(
		&self,

		sidecar_id:String,

		method:String,

		params:Value,
	) -> Result<(), CommonError> {
		vine::send_notification_to_sidecar(&sidecar_id, method, params)
			.await
			.map_err(|e| CommonError::IpcError(e.to_string()))
	}

	async fn send_request_to_sidecar(
		&self,

		sidecar_id:String,

		method:String,

		params:Value,

		timeout_ms:u64,
	) -> Result<Value, CommonError> {
		vine::send_request_to_sidecar(&sidecar_id, method, params, timeout_ms)
			.await
			.map_err(|e| CommonError::IpcError(e.to_string()))
	}
}

impl Requires<Arc<dyn IpcProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn IpcProvider + Send + Sync> { Arc::new(self.clone()) }
}

#[async_trait]
impl WorkspaceEditApplier for MountainEnvironment {
	// ... (Full implementation from previous synthesis using FileEditTypeDto
	// dispatch) ...
	async fn apply_workspace_edit(&self, edit_dto:WorkspaceEditDto) -> Result<bool, CommonError> {
		info!(
			"[Env WkspcEditApplier] Applying WorkspaceEdit: {} edits. Top-level metadata: {:?}",
			edit_dto.edits.len(),
			edit_dto.metadata.as_ref().and_then(|m| m.get("label"))
		);

		let doc_provider:Arc<dyn DocumentProvider + Send + Sync> = self.require();

		let fs_writer:Arc<dyn FsWriter + Send + Sync> = self.require();

		for (index, edit_entry_val) in edit_dto.edits.iter().enumerate() {
			trace!(
				"[Env WkspcEditApplier] Edit #{}: Type {:?}",
				index,
				edit_entry_val.get("_type").and_then(Value::as_u64)
			);

			let edit_type_num = edit_entry_val.get("_type").and_then(Value::as_u64);

			match edit_type_num.and_then(|v| serde_json::from_value(Value::from(v)).ok()) {
				Some(FileEditTypeDto::Text) | Some(FileEditTypeDto::Snippet) => {
					let text_op =
						serde_json::from_value::<WorkspaceTextEditDto>(edit_entry_val.clone()).map_err(|e| {
							CommonError::InvalidArg(
								"text_edit".to_string(),
								format!("Deserialize WorkspaceTextEditDto: {}", e),
							)
						})?;

					let target_uri = handlers::documents::parse_uri_from_components_param(
						&text_op.resource,
						"apply_edit_text",
						"resource",
						None,
					)?;

					let single_edit_op_val = text_op.edit;

					let rpc_model_content_change = json!({"range": single_edit_op_val.get("range").cloned(), "text": single_edit_op_val.get("text").cloned(), "eol": single_edit_op_val.get("eol").cloned() });

					let changes_array_val = Value::Array(vec![rpc_model_content_change]);

					let version_id_for_apply = text_op.version_id.map(|v| v as i64).unwrap_or(-1);

					info!("[Env WkspcEditApplier] Applying TextEdit to: {}", target_uri);

					doc_provider
						.apply_document_changes(
							target_uri.clone(),
							version_id_for_apply,
							changes_array_val,
							true,
							false,
							false,
						)
						.await?;
				},

				Some(FileEditTypeDto::File) => {
					let file_op =
						serde_json::from_value::<WorkspaceFileEditDto>(edit_entry_val.clone()).map_err(|e| {
							CommonError::InvalidArg(
								"file_edit".to_string(),
								format!("Deserialize WorkspaceFileEditDto: {}", e),
							)
						})?;

					let old_url_opt = file_op.old_uri.as_ref().and_then(|v| {
						handlers::documents::parse_uri_from_components_param(v, "apply_edit_file_old", "old_uri", None)
							.ok()
					});

					let new_url_opt = file_op.new_uri.as_ref().and_then(|v| {
						handlers::documents::parse_uri_from_components_param(v, "apply_edit_file_new", "new_uri", None)
							.ok()
					});

					let overwrite = file_op
						.options
						.as_ref()
						.and_then(|o| o.get("overwrite"))
						.and_then(Value::as_bool)
						.unwrap_or(false);

					let recursive = file_op
						.options
						.as_ref()
						.and_then(|o| o.get("recursive"))
						.and_then(Value::as_bool)
						.unwrap_or(false);

					let ignore_if_not_exists = file_op
						.options
						.as_ref()
						.and_then(|o| o.get("ignoreIfNotExists"))
						.and_then(Value::as_bool)
						.unwrap_or(false);

					if let (Some(old_uri), Some(new_uri)) = (old_url_opt.as_ref(), new_url_opt.as_ref()) {
						info!("[Env WkspcEditApplier] File Rename: {} -> {}", old_uri, new_uri);

						fs_writer
							.rename(&PathBuf::from(old_uri.path()), &PathBuf::from(new_uri.path()), overwrite)
							.await?;
					} else if let Some(new_uri) = new_url_opt.as_ref() {
						info!("[Env WkspcEditApplier] File Create: {}", new_uri);

						fs_writer
							.write_file(&PathBuf::from(new_uri.path()), Vec::new(), true, overwrite)
							.await?;
					} else if let Some(old_uri) = old_url_opt.as_ref() {
						info!("[Env WkspcEditApplier] File Delete: {}", old_uri);

						let path_to_delete = PathBuf::from(old_uri.path());

						if ignore_if_not_exists && !tokio::fs::try_exists(&path_to_delete).await.unwrap_or(false) {
							debug!("[Env WkspcEditApplier] Delete skipped (ignoreIfNotExists): {}", old_uri);
						} else {
							fs_writer.delete(&path_to_delete, recursive, false).await?;
						}
					} else {
						return Err(CommonError::InvalidArg(
							"file_edit".to_string(),
							"File op missing old/new URIs.".to_string(),
						));
					}
				},

				Some(FileEditTypeDto::Cell)
				| Some(FileEditTypeDto::CellReplace)
				| Some(FileEditTypeDto::CellMetadata)
				| Some(FileEditTypeDto::DocumentMetadata) => {
					warn!("[Env WkspcEditApplier] Notebook edit STUBBED: {:?}", edit_type_num);
				},

				_ => {
					warn!(
						"[Env WkspcEditApplier] Unknown _type in WorkspaceEditDto: {:?}. Edit: {:?}",
						edit_type_num, edit_entry_val
					);

					return Err(CommonError::InvalidArg(
						"edit_entry._type".to_string(),
						"Unknown edit _type".to_string(),
					));
				},
			}
		}

		Ok(true)
	}
}

impl Requires<Arc<dyn WorkspaceEditApplier + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn WorkspaceEditApplier + Send + Sync> { Arc::new(self.clone()) }
}

// --- LanguageFeatureProviderRegistry Implementation (Extensive) ---
// Macro to reduce boilerplate for simple provider methods
macro_rules! impl_lang_feat_provider_method {

    ($method_name:ident, $provider_type:expr, $params_builder:expr, $result_dto:ty, $rpc_name:expr, $timeout:expr) => {

        async fn $method_name(
            &self, document_uri: Url, language_id: String, position_dto: PositionDto,

 // Some methods take context_dto or range_dto instead of position_dto, or different set of args.

 // This macro is simplified for position-based ones. Complex ones need full impl.

 // For now, using a generic `Value` for extra_param that can be context or token.

            extra_param: Value, cancellation_token_id_val: Option<Value>
        ) -> Result<Option<$result_dto>, CommonError> {

            let providers = self.get_providers_for_document(document_uri.clone(), language_id, $provider_type).await?;

            if let Some(p_desc) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {

                let uri_dto = json!({"external": document_uri.to_string(), "$mid":1});

                let pos_dto_rpc = json!(position_dto);

                let token_dto_rpc = cancellation_token_id_val.unwrap_or(Value::Null);

 // The params_builder closure constructs the specific parameter array for the RPC call.

                let rpc_params = $params_builder(p_desc.handle, uri_dto, pos_dto_rpc, extra_param, token_dto_rpc);

                let rpc_method = format!("{}${}", ProxyTarget::ExtHostLanguageFeatures.target_prefix(), $rpc_name);

                match vine::send_request_to_sidecar(&p_desc.sidecar_id, rpc_method, rpc_params, $timeout).await {

                    Ok(v) if !v.is_null() => serde_json::from_value(v).map_err(|e| CommonError::IpcError(format!("Deserialize {}: {}", stringify!($result_dto), e))).map(Some),

                    Ok(_) => Ok(None),

                    Err(e) => Err(CommonError::IpcError(format!("RPC for {} failed: {}", $rpc_name, e))),

                }

            } else { Ok(None) }

        }

    };

 // Variant for methods that don't take position_dto but other primary args like query or range

    ($method_name:ident, $provider_type:expr, $main_arg_builder:expr, $params_builder_no_pos:expr, $result_dto:ty, $rpc_name:expr, $timeout:expr) => {

        async fn $method_name(
             // e.g. range_dto or query_string
			&self, document_uri: Url, language_id: String, main_arg: Value,

            cancellation_token_id_val: Option<Value>
        ) -> Result<Option<$result_dto>, CommonError> {

            let providers = self.get_providers_for_document(document_uri.clone(), language_id, $provider_type).await?;

            if let Some(p_desc) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {

                let uri_dto = json!({"external": document_uri.to_string(), "$mid":1});

                 // Closure to build main_arg for RPC
				let main_arg_rpc = $main_arg_builder(main_arg);

                let token_dto_rpc = cancellation_token_id_val.unwrap_or(Value::Null);

                let rpc_params = $params_builder_no_pos(p_desc.handle, uri_dto, main_arg_rpc, token_dto_rpc);

                let rpc_method = format!("{}${}", ProxyTarget::ExtHostLanguageFeatures.target_prefix(), $rpc_name);

                match vine::send_request_to_sidecar(&p_desc.sidecar_id, rpc_method, rpc_params, $timeout).await {

                    Ok(v) if !v.is_null() => serde_json::from_value(v).map_err(|e| CommonError::IpcError(format!("Deserialize {}: {}", stringify!($result_dto), e))).map(Some),

                    Ok(_) => Ok(None),

                    Err(e) => Err(CommonError::IpcError(format!("RPC for {} failed: {}", $rpc_name, e))),

                }

            } else { Ok(None) }

        }

    };

}

#[async_trait]
impl LanguageFeatureProviderRegistry for MountainEnvironment {
	async fn register_provider(
		&self,

		sidecar_id:String,

		provider_type_common:CommonProviderType,

		selector_dto_val:Value,

		extension_id_dto:Value,

		options_dto_val_opt:Option<ProviderOptionsDto>,
	) -> Result<u32, CommonError> {
		let app_state = self.get_app_state();

		let new_provider_handle = app_state.get_next_provider_handle();

		// let app_state_provider_type: AppStateLanguageProviderType =
		// No longer needed if ProviderRegistration uses
		// provider_type_common.into();

		// CommonProviderType
		info!(
			"[Env LangFeatReg Register] Type='{:?}', Handle={}, SidecarID='{}', ExtID='{:?}', OptionsIsSome={}",
			provider_type_common,
			new_provider_handle,
			sidecar_id,
			extension_id_dto.get("value"),
			options_dto_val_opt.is_some()
		);

		trace!(
			"[Env LangFeatReg Register] Selector: {:?}, Options: {:?}, Extension: {:?}",
			selector_dto_val, options_dto_val_opt, extension_id_dto
		);

		let new_registration = ProviderRegistration {
			handle:new_provider_handle,

			provider_type:provider_type_common,

			selector:selector_dto_val,

			sidecar_id,

			extension_id:extension_id_dto,

			options:options_dto_val_opt,
		};

		app_state
			.language_providers
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.insert(new_provider_handle, new_registration);

		Ok(new_provider_handle)
	}

	async fn unregister_provider(&self, provider_handle_to_remove:u32) -> Result<(), CommonError> {
		info!("[Env LangFeatReg Unregister] Handle: {}", provider_handle_to_remove);

		if self
			.get_app_state()
			.language_providers
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?
			.remove(&provider_handle_to_remove)
			.is_none()
		{
			warn!(
				"[Env LangFeatReg Unregister] Attempted unregister non-existent handle: {}",
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
			"[Env LangFeatReg GetProviders] For Doc='{}...', Lang='{}', Type='{:?}'",
			document_uri.path_segments().and_then(|s| s.last()).unwrap_or_default(),
			language_id,
			provider_type_common
		);

		let app_state_val = self.get_app_state();

		let providers_map_guard = app_state_val
			.language_providers
			.lock()
			.map_err(map_app_state_lock_error_to_common_error)?;

		let matching_providers_vec:Vec<ProviderDescription> = providers_map_guard
			.values()
			.filter(|reg| reg.provider_type == provider_type_common)
			.filter(|reg| handlers::config::match_document_selector(®.selector, &document_uri, &language_id))
			.map(|reg| {
				ProviderDescription {
					handle:reg.handle,

					sidecar_id:reg.sidecar_id.clone(),

					options:reg.options.as_ref().and_then(|o| serde_json::to_value(o).ok()),
				}
			})
			.collect();

		debug!(
			"[Env LangFeatReg GetProviders] Found {} matching {:?} for doc='{}...', lang='{}'",
			matching_providers_vec.len(),
			provider_type_common,
			document_uri.path_segments().and_then(|s| s.last()).unwrap_or_default(),
			language_id
		);

		Ok(matching_providers_vec)
	}

	// --- Invocation Methods ---
	async fn provide_hover(
		&self,

		document_uri:Url,

		language_id:String,

		// , _token_id_dto: Option<Value>
		position_dto:PositionDto,
	) -> Result<Option<HoverResultDto>, CommonError> {
		let providers = self
			.get_providers_for_document(document_uri.clone(), language_id, CommonProviderType::Hover)
			.await?;

		if let Some(p_desc) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {
			info!(
				"[Env LF Hover] Calling Cocoon provider (H:{}) for {}",
				p_desc.handle, document_uri
			);

			let rpc_params = json!([
				p_desc.handle,
				json!({"scheme": document_uri.scheme(), "path": document_uri.path(), "external": document_uri.to_string(), "$mid": 1}),
				json!(position_dto),
				Value::Null,
				Value::Null
			]);

			let rpc_method = format!("{}$provideHover", ProxyTarget::ExtHostLanguageFeatures.target_prefix());

			match vine::send_request_to_sidecar(&p_desc.sidecar_id, rpc_method, rpc_params, 10000).await {
				Ok(v) if !v.is_null() => {
					serde_json::from_value(v)
						.map_err(|e| CommonError::IpcError(format!("Deserialize HoverResultDto: {}", e)))
						.map(Some)
				},

				Ok(_) => Ok(None),

				Err(e) => Err(CommonError::IpcError(format!("RPC for hover: {}", e))),
			}
		} else {
			Ok(None)
		}
	}

	async fn provide_completions(
		&self,

		document_uri:Url,

		language_id:String,

		position_dto:PositionDto,

		context_dto:CompletionContextDto,

		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<SuggestResultDto>, CommonError> {
		let providers = self
			.get_providers_for_document(document_uri.clone(), language_id, CommonProviderType::Completion)
			.await?;

		if let Some(p_desc) = providers.iter().find(|p| p.sidecar_id.starts_with("cocoon")) {
			let rpc_params = json!([
				p_desc.handle,
				json!({"external": document_uri.to_string(), "$mid":1}),
				json!(position_dto),
				json!(context_dto),
				cancellation_token_id_val.unwrap_or(Value::Null)
			]);

			let rpc_method = format!(
				"{}$provideCompletionItems",
				ProxyTarget::ExtHostLanguageFeatures.target_prefix()
			);

			match vine::send_request_to_sidecar(&p_desc.sidecar_id, rpc_method, rpc_params, 15000).await {
				Ok(v) if !v.is_null() => {
					serde_json::from_value(v)
						.map_err(|e| CommonError::IpcError(format!("Deserialize SuggestResultDto: {}", e)))
						.map(Some)
				},

				Ok(_) => Ok(None),

				Err(e) => Err(CommonError::IpcError(format!("RPC for completions: {}", e))),
			}
		} else {
			Ok(None)
		}
	}

	async fn resolve_completion_item_for_list(
		&self,

		list_cache_id:u32,

		item_to_resolve_dto:Value,

		cancellation_token_id_val:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		let provider_reg = self
			.get_provider_registration_from_handle(list_cache_id, CommonLangProviderType::Completion)
			.await?;

		let rpc_params = json!([
			list_cache_id,
			item_to_resolve_dto,
			cancellation_token_id_val.unwrap_or(Value::Null)
		]);

		let rpc_method = format!("{}$resolveCompletionItem", ProxyTarget::ExtHostLanguageFeatures.target_prefix());

		match vine::send_request_to_sidecar(&provider_reg.sidecar_id, rpc_method, rpc_params, 5000).await {
			Ok(v) if !v.is_null() => Ok(Some(v)),

			Ok(_) => Ok(None),

			Err(e) => Err(CommonError::IpcError(format!("RPC for resolveCompletionItem: {}", e))),
		}
	}

	// ... (provide_definition, provide_declaration, provide_implementation,

	// provide_type_definition - similar pattern to provide_hover) ...
	async fn provide_definition(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<LocationLinkDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_declaration(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<LocationLinkDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_implementation(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<LocationLinkDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_type_definition(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<LocationLinkDto>>, CommonError> {
		// ...
		Ok(None)
	}

	// ... (provide_code_actions, resolve_code_action - similar pattern, ensure DTOs
	// match) ...
	async fn provide_code_actions(
		&self,

		doc_uri:Url,

		lang_id:String,

		range_or_sel:Value,

		ctx_dto:CodeActionContextDto,

		token_val:Option<Value>,
	) -> Result<Option<CodeActionListDto>, CommonError> {
		// ...
		Ok(None)
	}

	async fn resolve_code_action(
		&self,

		list_cache_id:u32,

		_sid:String,

		action_dto:Value,

		token_val:Option<Value>,
	) -> Result<Option<CodeActionDto>, CommonError> {
		// ...
		Ok(None)
	}

	// ... (provide_code_lenses, resolve_code_lens - similar pattern) ...
	async fn provide_code_lenses(
		&self,

		doc_uri:Url,

		lang_id:String,

		token_val:Option<Value>,
	) -> Result<Option<CodeLensListDto>, CommonError> {
		// ...
		Ok(None)
	}

	async fn resolve_code_lens(
		&self,

		list_cache_id:u32,

		_sid:String,

		lens_dto:Value,

		token_val:Option<Value>,
	) -> Result<Option<CodeLensDto>, CommonError> {
		// ...
		Ok(None)
	}

	// ... (provide_document_symbols, provide_workspace_symbols,

	// provide_signature_help - similar pattern) ...
	async fn provide_document_symbols(
		&self,

		doc_uri:Url,

		lang_id:String,

		token_val:Option<Value>,
	) -> Result<Option<Vec<DocumentSymbolDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_workspace_symbols(
		&self,

		query:String,

		token_val:Option<Value>,
	) -> Result<Option<Vec<WorkspaceSymbolDto>>, CommonError> {
		// For workspace symbols, query all providers of this type
		Ok(None)
	}

	async fn provide_signature_help(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		ctx_dto:SignatureHelpContextDto,

		token_val:Option<Value>,
	) -> Result<Option<SignatureHelpResultDto>, CommonError> {
		// ...
		Ok(None)
	}

	// ... (provide_document_formatting_edits, provide_document_highlights,

	// provide_document_links, resolve_document_link - similar pattern) ...
	async fn provide_document_formatting_edits(
		&self,

		doc_uri:Url,

		lang_id:String,

		opts_dto:FormattingOptionsDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<TextEditDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_document_highlights(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<DocumentHighlightDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_document_links(
		&self,

		doc_uri:Url,

		lang_id:String,

		token_val:Option<Value>,
	) -> Result<Option<LinksListDto>, CommonError> {
		// ...
		Ok(None)
	}

	async fn resolve_document_link(
		&self,

		list_cache_id:u32,

		_sid:String,

		link_dto:Value,

		token_val:Option<Value>,
	) -> Result<Option<LinkDto>, CommonError> {
		// ...
		Ok(None)
	}

	// ... (provide_references, prepare_rename, provide_rename_edits - similar
	// pattern) ...
	async fn provide_references(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		ctx_dto:Value,

		token_val:Option<Value>,
	) -> Result<Option<Vec<LocationLinkDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn prepare_rename(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		token_val:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_rename_edits(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		new_name:String,

		token_val:Option<Value>,
	) -> Result<Option<WorkspaceEditDto>, CommonError> {
		// ...
		Ok(None)
	}

	// ... (provide_folding_ranges, provide_selection_ranges,

	// provide_linked_editing_ranges - similar pattern) ...
	async fn provide_folding_ranges(
		&self,

		doc_uri:Url,

		lang_id:String,

		ctx_dto:Value,

		token_val:Option<Value>,
	) -> Result<Option<Vec<FoldingRangeDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_selection_ranges(
		&self,

		doc_uri:Url,

		lang_id:String,

		positions_dto:Vec<PositionDto>,

		token_val:Option<Value>,
	) -> Result<Option<Vec<SelectionRangeDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_linked_editing_ranges(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		token_val:Option<Value>,
	) -> Result<Option<LinkedEditingRangesDto>, CommonError> {
		// ...
		Ok(None)
	}

	// ... (provide_document_semantic_tokens,

	// provide_document_semantic_tokens_edits,

	// provide_document_range_semantic_tokens - similar pattern) ...
	async fn provide_document_semantic_tokens(
		&self,

		doc_uri:Url,

		lang_id:String,

		prev_id:Option<String>,

		token_val:Option<Value>,
	) -> Result<Option<SemanticTokensDto>, CommonError> {
		// Returns SemanticTokensDto
		Ok(None)
	}

	async fn provide_document_semantic_tokens_edits(
		&self,

		doc_uri:Url,

		lang_id:String,

		prev_id:String,

		token_val:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		// Returns Value (SemanticTokensDto | SemanticTokensEditsDto)
		Ok(None)
	}

	async fn provide_document_range_semantic_tokens(
		&self,

		doc_uri:Url,

		lang_id:String,

		range_dto:RangeDto,

		token_val:Option<Value>,
	) -> Result<Option<SemanticTokensDto>, CommonError> {
		// ...
		Ok(None)
	}

	// ... (prepare_call_hierarchy, provide_call_hierarchy_incoming_calls,

	// provide_call_hierarchy_outgoing_calls - similar pattern) ...
	async fn prepare_call_hierarchy(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<HierarchyItemDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_call_hierarchy_incoming_calls(
		&self,

		_sid:String,

		item_dto:HierarchyItemDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<IncomingCallDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_call_hierarchy_outgoing_calls(
		&self,

		_sid:String,

		item_dto:HierarchyItemDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<OutgoingCallDto>>, CommonError> {
		// ...
		Ok(None)
	}

	// ... (prepare_type_hierarchy, provide_type_hierarchy_supertypes,

	// provide_type_hierarchy_subtypes - similar pattern) ...
	async fn prepare_type_hierarchy(
		&self,

		doc_uri:Url,

		lang_id:String,

		pos_dto:PositionDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<HierarchyItemDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_type_hierarchy_supertypes(
		&self,

		_sid:String,

		item_dto:HierarchyItemDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<HierarchyItemDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn provide_type_hierarchy_subtypes(
		&self,

		_sid:String,

		item_dto:HierarchyItemDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<HierarchyItemDto>>, CommonError> {
		// ...
		Ok(None)
	}

	// ... (provide_inlay_hints, resolve_inlay_hint - similar pattern) ...
	async fn provide_inlay_hints(
		&self,

		doc_uri:Url,

		lang_id:String,

		range_dto:RangeDto,

		token_val:Option<Value>,
	) -> Result<Option<Vec<InlayHintDto>>, CommonError> {
		// ...
		Ok(None)
	}

	async fn resolve_inlay_hint(
		&self,

		provider_handle:u32,

		_sid:String,

		hint_dto_val:Value,

		token_val:Option<Value>,
	) -> Result<Option<InlayHintDto>, CommonError> {
		// ...
		Ok(None)
	}

	// Implement other formatting methods (range, on-type)
	async fn provide_document_range_formatting_edits(
		&self,

		_doc_uri:Url,

		_lang_id:String,

		_range_dto:RangeDto,

		_options_dto:FormattingOptionsDto,

		_token_val:Option<Value>,
	) -> Result<Option<Vec<TextEditDto>>, CommonError> {
		warn!("provide_document_range_formatting_edits STUBBED");

		Ok(None)
	}

	async fn provide_on_type_formatting_edits(
		&self,

		_doc_uri:Url,

		_lang_id:String,

		_position_dto:PositionDto,

		_ch:String,

		_options_dto:FormattingOptionsDto,

		_token_val:Option<Value>,
	) -> Result<Option<Vec<TextEditDto>>, CommonError> {
		warn!("provide_on_type_formatting_edits STUBBED");

		Ok(None)
	}
}

impl Requires<Arc<dyn LanguageFeatureProviderRegistry + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn LanguageFeatureProviderRegistry + Send + Sync> { Arc::new(self.clone()) }
}
