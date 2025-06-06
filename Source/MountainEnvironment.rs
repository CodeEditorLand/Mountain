
// Primary Focus: Defines the Mountain application's specific environment,
// implementing various provider traits to interact with the application's state
// and services.

use std::{
	collections::HashMap,
	ffi::OsStr,
	path::{Path, PathBuf},
	sync::{Arc, MutexGuard as StdMutexGuard},
	time::Duration as StdDuration,
};

use Common::{
	CommandEffect::CommandExecutor,
	ConfigEffect::{
		ConfigInspector,
		ConfigProvider,
		ConfigurationScope,
		ConfigurationTarget,
		IConfigurationInitDataDto,
		IConfigurationOverrides,
		InspectResultData,
	},
	DiagnosticsEffect::{DiagnosticsManager, MarkerDataDto as CommonMarkerDataDto},
	DocumentEffect::{DocEventParams, DocumentProvider},
	Environment::{Environment, Requires},
	Errors::CommonError,
	FsEffect::{FileSystemProviderCapabilities, FileSystemStat, FileType as CommonFileType, FsReader, FsWriter},
	IpcEffect::{IpcProvider, ProxyConfiguration as ProxyTarget}, // Renamed from ProxyTarget for clarity
	LanguageFeatureEffect::{
		self, // For DTOs
		CodeActionContextDto,
		CodeActionDto,
		CodeActionListDto,
		CodeLensDto,
		CodeLensListDto,
		CompletionContextDto,
		DocumentHighlightDto,
		DocumentSymbolDto,
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
		WorkspaceCellEditDto, // Added from workspace_provider.rs for WorkspaceEditApplier
		WorkspaceEditDto,
		WorkspaceFileEditDto,
		WorkspaceSymbolDto,
		WorkspaceTextEditDto,
	},
	OutputEffect::OutputChannelManager,
	SecretsEffect::SecretsProvider,
	StorageEffect::StorageProvider,
	UiEffect::{
		self, // For DTOs
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
	WorkspaceEffect::{WorkspaceEditApplier, WorkspaceProvider},
};
use async_trait::async_trait;
use chrono::Utc;
use log::{LevelFilter as LogLevelFilter, debug, error, info, trace, warn};
use serde::Serialize;
use serde_json::{Map as JsonMap, Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime as TauriRuntime, State as TauriState, Window, Wry};
use tokio::{
	fs,
	io::AsyncWriteExt,
	sync::oneshot as TokioOneshot,
	time::{Duration as TokioDuration, timeout as TokioTimeout},
};
use url::Url;
use uuid::Uuid;

use crate::AppState::{HierarchySessionContext, ProviderRegistration as AppStateProviderRegistration};
use crate::{
	AppState,            // Assuming AppState is in the crate root or accessible via `crate::`
	Handlers,            // Assuming Handlers module is in the crate root
	Runtime::AppRuntime, // Assuming AppRuntime is in Runtime module
	Vine,                // Assuming Vine module is in the crate root
}; // Specific AppState items

// Private helper functions 
mod InternalUtils {
	use super::*; // Make parent's imports available

	pub fn MapAppStateLockErrorToCommonError<T>(Error:std::sync::PoisonError<StdMutexGuard<'_, T>>) -> CommonError {
		let ErrorMessage = format!("[MountainEnvironment Utils] Failed to lock AppState section: {}", Error);
		error!("{}", ErrorMessage);
		CommonError::StateLock { Context:ErrorMessage }
	}

	pub fn MapIoErrorToCommonError(Error:std::io::Error, Path:PathBuf, Operation:&'static str) -> CommonError {
		warn!(
			"[MountainEnvironment Utils IOError] FS op '{}' on '{}' failed: {}",
			Operation,
			Path.display(),
			Error
		);
		match Error.kind() {
			std::io::ErrorKind::NotFound => CommonError::FsNotFound(Path),
			std::io::ErrorKind::PermissionDenied => CommonError::FsPermissionDenied { Path, Reason:Error.to_string() },
			std::io::ErrorKind::AlreadyExists => CommonError::FsFileExists(Path),
			std::io::ErrorKind::IsADirectory => CommonError::FsIsADirectory(Path),
			std::io::ErrorKind::NotADirectory => CommonError::FsNotADirectory(Path),
			_ => {
				match Operation {
					"read" | "read_doc_open" => CommonError::FsRead { Path, Description:Error.to_string() },
					"write" | "write_doc_save" | "write_doc_save_as" | "create_file" => {
						CommonError::FsWrite { Path, Description:Error.to_string() }
					},
					"stat" | "copy_stat" | "delete_stat_check" | "rename_target_stat" => {
						CommonError::FsStat { Path, Description:Error.to_string() }
					},
					"mkdir" | "mkdir_all" | "mkdir_parent" | "mkdir_parent_rename" | "mkdir_parent_copy" => {
						CommonError::FsMkdir { Path, Description:Error.to_string() }
					},
					"delete" => CommonError::FsDelete { Path, Description:Error.to_string() },
					"rename" => {
						CommonError::FsRename { Source:Path, Target:PathBuf::new(), Description:Error.to_string() }
					},
					"copy" => CommonError::FsCopy { Source:Path, Target:PathBuf::new(), Description:Error.to_string() },
					"readdir" | "readdir_next" | "readdir_next_entry" => {
						CommonError::FsReadDir { Path, Description:Error.to_string() }
					},
					_ => {
						CommonError::Unknown {
							Description:format!(
								"Unknown FS Op '{}' on '{}' failed: {}",
								Operation,
								Path.display(),
								Error
							),
						}
					},
				}
			},
		}
	}

	pub fn DetectLanguageIdentifierFromFilePath(PathReference:&Path) -> String {
		match PathReference.extension().and_then(OsStr::to_str) {
			Some("js") | Some("mjs") | Some("cjs") => "javascript".to_string(),
			Some("jsx") => "javascriptreact".to_string(),
			Some("ts") => "typescript".to_string(),
			Some("tsx") => "typescriptreact".to_string(),
			// ... (all other mappings from the original file)
			_ => "plaintext".to_string(),
		}
	}

	pub fn DetectFileEncodingFromBytes(_ContentBytes:&[u8]) -> String {
		"utf8".to_string() // Placeholder
	}

	pub async fn IsPathAllowedForFilesystemAccess<Runtime:TauriRuntime>(
		AppHandleReference:&AppHandle<Runtime>,
		PathToCheck:&Path,
	) -> Result<(), CommonError> {
		trace!("[MountainEnvironment SecCheck] Verifying path: {}", PathToCheck.display());
		let PathToCheckOwned = PathToCheck.to_path_buf();

		let CanonicalPathResult = tokio::task::spawn_blocking(move || -> Result<PathBuf, std::io::Error> {
			match std::fs::canonicalize(&PathToCheckOwned) {
				Ok(p) => Ok(p),
				Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
					PathToCheckOwned
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
						.map(|cp| cp.join(PathToCheckOwned.file_name().unwrap_or_else(|| std::ffi::OsStr::new(""))))
				},
				Err(e) => Err(e),
			}
		})
		.await;

		let CanonicalPathToCheck = match CanonicalPathResult {
			Ok(Ok(p)) => p,
			Ok(Err(IoError)) => {
				return Err(CommonError::FsPermissionDenied {
					Path:PathToCheck.to_path_buf(),
					Reason:format!("Path canonicalization failed: {}. Path: '{}'", IoError, PathToCheck.display()),
				});
			},
			Err(JoinError) => {
				return Err(CommonError::FsPermissionDenied {
					Path:PathToCheck.to_path_buf(),
					Reason:format!(
						"Task join error during canonicalization: {}. Path: '{}'",
						JoinError,
						PathToCheck.display()
					),
				});
			},
		};
		trace!(
			"[MountainEnvironment SecCheck] Canonical path for '{}': '{}'",
			PathToCheck.display(),
			CanonicalPathToCheck.display()
		);

		let mut AllowedRootPaths:Vec<PathBuf> = Vec::new();
		let AppStateInstance = AppHandleReference.state::<AppState>();

		let FoldersGuard = AppStateInstance
			.WorkspaceFolders
			.lock()
			.map_err(MapAppStateLockErrorToCommonError)?;
		for Folder in FoldersGuard.iter() {
			if Folder.Uri.scheme() == "file" {
				if let Ok(CanonicalFolderPath) = std::fs::canonicalize(PathBuf::from(Folder.Uri.path())) {
					AllowedRootPaths.push(CanonicalFolderPath);
				} else {
					warn!(
						"[MountainEnvironment SecCheck] Failed to canonicalize workspace folder: {}",
						Folder.Uri
					);
				}
			}
		}
		drop(FoldersGuard);

		let PathResolver = AppHandleReference.path_resolver();
		for DirOption in [
			PathResolver.app_config_dir(),
			PathResolver.app_data_dir(),
			PathResolver.app_log_dir(),
		] {
			if let Some(DirectoryPath) = DirOption {
				if let Ok(CanonicalAppPath) = std::fs::canonicalize(&DirectoryPath) {
					AllowedRootPaths.push(CanonicalAppPath);
				} else {
					warn!(
						"[MountainEnvironment SecCheck] Failed to canonicalize app system dir: {}",
						DirectoryPath.display()
					);
				}
			}
		}

		if let Ok(GlobalMementoPathCanonical) = std::fs::canonicalize(&AppStateInstance.GlobalMementoPath) {
			AllowedRootPaths.push(GlobalMementoPathCanonical.clone());
			if let Some(ParentPath) = GlobalMementoPathCanonical.parent() {
				if let Ok(ParentCanonical) = std::fs::canonicalize(ParentPath) {
					AllowedRootPaths.push(ParentCanonical);
				}
			}
		}
		if let Some(ref WorkspaceMementoPathOption) = *AppStateInstance
			.WorkspaceMementoPath
			.lock()
			.map_err(MapAppStateLockErrorToCommonError)?
		{
			if let Some(ref WorkspaceMementoPath) = WorkspaceMementoPathOption {
				if let Ok(CanonicalWorkspaceMementoPath) = std::fs::canonicalize(WorkspaceMementoPath) {
					AllowedRootPaths.push(CanonicalWorkspaceMementoPath.clone());
					if let Some(ParentPath) = CanonicalWorkspaceMementoPath.parent() {
						if let Ok(ParentCanonical) = std::fs::canonicalize(ParentPath) {
							AllowedRootPaths.push(ParentCanonical);
						}
					}
				}
			}
		}

		let IsAllowed = AllowedRootPaths
			.iter()
			.any(|RootPath| CanonicalPathToCheck == *RootPath || CanonicalPathToCheck.starts_with(RootPath));
		if IsAllowed {
			trace!("[MountainEnvironment SecCheck] ALLOWED: '{}'", PathToCheck.display());
			Ok(())
		} else {
			warn!(
				"[MountainEnvironment SecCheck] DENIED: '{}' (canonical: '{}'). Not in roots: {:?}",
				PathToCheck.display(),
				CanonicalPathToCheck.display(),
				AllowedRootPaths
			);
			Err(CommonError::FsPermissionDenied {
				Path:PathToCheck.to_path_buf(),
				Reason:"Path outside allowed workspace/app data folders.".to_string(),
			})
		}
	}
}

/// `MountainEnvironment` struct provides the application-specific environment
/// implementation. It holds a handle to the Tauri application, allowing it to
/// access application state and services.
#[derive(Clone)]
pub struct MountainEnvironment {
	AppHandle:AppHandle<Wry>,
}

impl MountainEnvironment {
	/// Creates a new `MountainEnvironment`.
	pub fn New(AppHandle:AppHandle<Wry>) -> Self {
		info!("[MountainEnvironment] New instance created.");
		Self { AppHandle }
	}

	/// Gets the application state from the Tauri application handle.
	fn GetAppState(&self) -> TauriState<'_, AppState> { self.AppHandle.state::<AppState>() }

	/// Gets the capabilities of the file provider for the 'file' scheme.
	pub fn GetFileProviderCapabilities(&self) -> u32 {
		let mut Capabilities = FileSystemProviderCapabilities::FileReadWrite as u32
			| FileSystemProviderCapabilities::FileOpenReadWriteLock as u32
			| FileSystemProviderCapabilities::FileFolderCopy as u32;

		if std::env::consts::OS != "windows" {
			Capabilities |= FileSystemProviderCapabilities::PathCaseSensitive as u32;
		}
		debug!(
			"[MountainEnvironment] File provider capabilities for 'file' scheme determined: {}",
			Capabilities
		);
		Capabilities
	}

	/// Retrieves a provider registration from the application state based on
	/// its handle and expected type.
	async fn GetProviderRegistrationFromHandle(
		&self,
		Handle:u32,
		ExpectedType:CommonProviderType,
	) -> Result<AppStateProviderRegistration, CommonError> {
		let AppStateInstance = self.GetAppState();
		let ProvidersGuard = AppStateInstance
			.LanguageProviders
			.lock()
			.map_err(InternalUtils::MapAppStateLockErrorToCommonError)?;

		ProvidersGuard
			.get(&Handle)
			.filter(|Registration| Registration.ProviderType == ExpectedType)
			.cloned()
			.ok_or_else(|| {
				CommonError::InvalidArg {
					ArgumentName:"handle".to_string(),
					Reason:format!("No matching {:?} provider for handle {}", ExpectedType, Handle),
				}
			})
	}

	async fn GetSidecarIdentifierForHierarchySession(
		&self,
		SessionIdentifier:&str,
		ExpectedType:CommonProviderType,
	) -> Result<String, CommonError> {
		let AppStateInstance = self.GetAppState();
		let SessionsGuard = AppStateInstance
			.ActiveHierarchySessions
			.lock()
			.map_err(InternalUtils::MapAppStateLockErrorToCommonError)?;

		if let Some(SessionContext) = SessionsGuard.get(SessionIdentifier) {
			if SessionContext.ProviderType == ExpectedType {
				Ok(SessionContext.OriginalSidecarIdentifier.clone())
			} else {
				Err(CommonError::InvalidArg {
					ArgumentName:"session_id".to_string(),
					Reason:format!("Session {} is not for {:?} hierarchy", SessionIdentifier, ExpectedType),
				})
			}
		} else {
			warn!(
				"[MountainEnvironment Hierarchy] Session ID '{}' not found in active sessions. Defaulting to \
				 'cocoon-main'. This may be incorrect.",
				SessionIdentifier
			);
			Ok("cocoon-main".to_string())
		}
	}
}

impl Environment for MountainEnvironment {}

// --- FsReader Implementation ---
#[async_trait]
impl FsReader for MountainEnvironment {
	async fn ReadFile(&self, Path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		InternalUtils::IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		trace!("[MountainEnvironment FsReader] Reading file: {}", Path.display());
		fs::read(Path)
			.await
			.map_err(|IoError| InternalUtils::MapIoErrorToCommonError(IoError, Path.clone(), "read"))
	}

	async fn StatFile(&self, Path:&PathBuf) -> Result<FileSystemStat, CommonError> {
		InternalUtils::IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		trace!("[MountainEnvironment FsReader] Stating file/directory: {}", Path.display());
		match tokio::fs::metadata(Path).await {
			Ok(Metadata) => {
				let mut FileTypeFlags = 0_u8;
				if Metadata.is_file() {
					FileTypeFlags |= CommonFileType::File as u8;
				}
				if Metadata.is_dir() {
					FileTypeFlags |= CommonFileType::Directory as u8;
				}
				if Metadata.is_symlink() {
					FileTypeFlags |= CommonFileType::SymbolicLink as u8;
				}

				let GetMilliTimestampFromSystemTime = |SystemTimeResult:Result<std::time::SystemTime, _>| -> u64 {
					SystemTimeResult
						.ok()
						.and_then(|Time| Time.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
						.map_or(0, |Duration| Duration.as_millis() as u64)
				};

				Ok(FileSystemStat {
					FileType:FileTypeFlags,
					CreationTime:GetMilliTimestampFromSystemTime(Metadata.created()),
					ModificationTime:GetMilliTimestampFromSystemTime(Metadata.modified()),
					Size:Metadata.len(),
					Permissions:None, // Permissions are not consistently available or easy to map cross-platform here
				})
			},
			Err(IoError) => Err(InternalUtils::MapIoErrorToCommonError(IoError, Path.clone(), "stat")),
		}
	}

	async fn ReadDirectory(&self, Path:&PathBuf) -> Result<Vec<(String, CommonFileType)>, CommonError> {
		InternalUtils::IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		debug!("[MountainEnvironment FsReader] Reading directory contents: {}", Path.display());
		let mut EntriesVector:Vec<(String, CommonFileType)> = Vec::new();
		let mut DirEntriesStream = fs::read_dir(Path)
			.await
			.map_err(|IoError| InternalUtils::MapIoErrorToCommonError(IoError, Path.clone(), "readdir"))?;

		while let Some(DirEntryResult) = DirEntriesStream
			.next_entry()
			.await
			.map_err(|IoError| InternalUtils::MapIoErrorToCommonError(IoError, Path.clone(), "readdir_next_entry"))?
		{
			let FileNameOsString = DirEntryResult.file_name();
			let FileNameString = FileNameOsString.to_string_lossy().into_owned();
			match DirEntryResult.file_type().await {
				Ok(TokioFileType) => {
					let CommonFileTypeValue = if TokioFileType.is_dir() {
						CommonFileType::Directory
					} else if TokioFileType.is_file() {
						CommonFileType::File
					} else if TokioFileType.is_symlink() {
						CommonFileType::SymbolicLink
					} else {
						CommonFileType::Unknown
					};
					EntriesVector.push((FileNameString, CommonFileTypeValue));
				},
				Err(ErrorFileType) => {
					warn!(
						"[MountainEnvironment FsReader] Failed to get file type for entry '{}' in directory '{}': {}. \
						 Marking as Unknown.",
						FileNameString,
						Path.display(),
						ErrorFileType
					);
					EntriesVector.push((FileNameString, CommonFileType::Unknown));
				},
			}
		}
		Ok(EntriesVector)
	}
}

impl Requires<Arc<dyn FsReader + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn FsReader + Send + Sync> { Arc::new(self.clone()) }
}

// --- FsWriter Implementation ---
#[async_trait]
impl FsWriter for MountainEnvironment {
	async fn WriteFile(
		&self,
		Path:&PathBuf,
		ContentBytes:Vec<u8>,
		CreateIfNotExists:bool,
		OverwriteIfExists:bool,
	) -> Result<(), CommonError> {
		InternalUtils::IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		info!(
			"[MountainEnvironment FsWriter] Writing file: path='{}', len={}, create={}, overwrite={}",
			Path.display(),
			ContentBytes.len(),
			CreateIfNotExists,
			OverwriteIfExists
		);
		let PathExists = fs::try_exists(Path).await.unwrap_or(false);
		if PathExists && !OverwriteIfExists {
			return Err(CommonError::FsFileExists(Path.clone()));
		}
		if !PathExists && !CreateIfNotExists {
			return Err(CommonError::FsNotFound(Path.clone()));
		}
		if let Some(ParentDirPath) = Path.parent() {
			if !fs::try_exists(ParentDirPath).await.unwrap_or(false) {
				if CreateIfNotExists {
					fs::create_dir_all(ParentDirPath).await.map_err(|IoError| {
						InternalUtils::MapIoErrorToCommonError(
							IoError,
							ParentDirPath.to_path_buf(),
							"mkdir_parent_for_write",
						)
					})?;
				} else {
					return Err(CommonError::FsNotFound(ParentDirPath.to_path_buf()));
				}
			}
		}
		fs::write(Path, &ContentBytes)
			.await
			.map_err(|IoError| InternalUtils::MapIoErrorToCommonError(IoError, Path.clone(), "write"))?;
		Ok(())
	}

	async fn CreateDirectory(&self, Path:&PathBuf, RecursiveCreate:bool) -> Result<(), CommonError> {
		InternalUtils::IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		info!(
			"[MountainEnvironment FsWriter] Creating directory: path='{}', recursive={}",
			Path.display(),
			RecursiveCreate
		);
		if RecursiveCreate {
			fs::create_dir_all(Path)
				.await
				.map_err(|IoError| InternalUtils::MapIoErrorToCommonError(IoError, Path.clone(), "mkdir_all"))?;
		} else {
			fs::create_dir(Path)
				.await
				.map_err(|IoError| InternalUtils::MapIoErrorToCommonError(IoError, Path.clone(), "mkdir"))?;
		}
		Ok(())
	}

	async fn CreateFile(&self, Path:&PathBuf) -> Result<(), CommonError> {
		InternalUtils::IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		info!("[MountainEnvironment FsWriter] Creating empty file: {}", Path.display());
		if fs::try_exists(Path).await.unwrap_or(false) {
			return Err(CommonError::FsFileExists(Path.clone()));
		}
		if let Some(ParentDir) = Path.parent() {
			if !fs::try_exists(ParentDir).await.unwrap_or(false) {
				fs::create_dir_all(ParentDir).await.map_err(|IoError| {
					InternalUtils::MapIoErrorToCommonError(
						IoError,
						ParentDir.to_path_buf(),
						"mkdir_parent_for_create_file",
					)
				})?;
			}
		}
		fs::File::create(Path)
			.await
			.map(|_| ())
			.map_err(|IoError| InternalUtils::MapIoErrorToCommonError(IoError, Path.clone(), "create_file"))?;
		Ok(())
	}

	async fn Delete(&self, Path:&PathBuf, RecursiveDelete:bool, UseOsTrash:bool) -> Result<(), CommonError> {
		InternalUtils::IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		info!(
			"[MountainEnvironment FsWriter] Deleting: path='{}', recursive={}, useTrash={}",
			Path.display(),
			RecursiveDelete,
			UseOsTrash
		);
		if UseOsTrash {
			warn!("[MountainEnvironment FsWriter] 'useTrash=true' for delete is STUBBED, performing permanent delete.");
		}
		match fs::metadata(Path).await {
			Ok(Metadata) => {
				let DeleteOperationResult = if Metadata.is_dir() {
					if RecursiveDelete {
						fs::remove_dir_all(Path).await
					} else {
						fs.remove_dir(Path).await
					}
				} else {
					fs::remove_file(Path).await
				};
				DeleteOperationResult
					.map_err(|IoError| InternalUtils::MapIoErrorToCommonError(IoError, Path.clone(), "delete"))?;
				Ok(())
			},
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
				debug!(
					"[MountainEnvironment FsWriter] Path '{}' not found for deletion (idempotent success).",
					Path.display()
				);
				Ok(())
			},
			Err(IoError) => {
				Err(InternalUtils::MapIoErrorToCommonError(
					IoError,
					Path.clone(),
					"delete_stat_check",
				))
			},
		}
	}

	async fn Rename(
		&self,
		SourcePath:&PathBuf,
		TargetPath:&PathBuf,
		OverwriteIfExists:bool,
	) -> Result<(), CommonError> {
		InternalUtils::IsPathAllowedForFilesystemAccess(&self.AppHandle, SourcePath).await?;
		InternalUtils::IsPathAllowedForFilesystemAccess(&self.AppHandle, TargetPath).await?;
		info!(
			"[MountainEnvironment FsWriter] Renaming: from='{}', to='{}', overwrite={}",
			SourcePath.display(),
			TargetPath.display(),
			OverwriteIfExists
		);
		if !fs::try_exists(SourcePath).await.unwrap_or(false) {
			return Err(CommonError::FsNotFound(SourcePath.clone()));
		}
		if !OverwriteIfExists && fs::try_exists(TargetPath).await.unwrap_or(false) {
			return Err(CommonError::FsFileExists(TargetPath.clone()));
		}
		if OverwriteIfExists && fs::try_exists(TargetPath).await.unwrap_or(false) {
			debug!(
				"[MountainEnvironment FsWriter] Rename: Overwriting target by first deleting '{}'",
				TargetPath.display()
			);
			let TargetMetadata = fs::metadata(TargetPath).await.map_err(|IoError| {
				InternalUtils::MapIoErrorToCommonError(IoError, TargetPath.clone(), "rename_target_stat_for_overwrite")
			})?;
			self.Delete(TargetPath, TargetMetadata.is_dir(), false).await?;
		}
		if let Some(TargetParentDir) = TargetPath.parent() {
			if !fs::try_exists(TargetParentDir).await.unwrap_or(false) {
				fs::create_dir_all(TargetParentDir).await.map_err(|IoError| {
					InternalUtils::MapIoErrorToCommonError(
						IoError,
						TargetParentDir.to_path_buf(),
						"mkdir_parent_for_rename",
					)
				})?;
			}
		}
		fs::rename(SourcePath, TargetPath)
			.await
			.map_err(|IoError| InternalUtils::MapIoErrorToCommonError(IoError, SourcePath.clone(), "rename"))?;
		Ok(())
	}

	async fn Copy(&self, SourcePath:&PathBuf, TargetPath:&PathBuf, OverwriteIfExists:bool) -> Result<(), CommonError> {
		InternalUtils::IsPathAllowedForFilesystemAccess(&self.AppHandle, SourcePath).await?;
		InternalUtils::IsPathAllowedForFilesystemAccess(&self.AppHandle, TargetPath).await?;
		info!(
			"[MountainEnvironment FsWriter] Copying: from='{}', to='{}', overwrite={}",
			SourcePath.display(),
			TargetPath.display(),
			OverwriteIfExists
		);
		if !fs::try_exists(SourcePath).await.unwrap_or(false) {
			return Err(CommonError::FsNotFound(SourcePath.clone()));
		}
		if !OverwriteIfExists && fs::try_exists(TargetPath).await.unwrap_or(false) {
			return Err(CommonError::FsFileExists(TargetPath.clone()));
		}
		let SourceMetadata = fs::metadata(SourcePath).await.map_err(|IoError| {
			InternalUtils::MapIoErrorToCommonError(IoError, SourcePath.clone(), "copy_source_stat")
		})?;
		if SourceMetadata.is_dir() {
			error!(
				"[MountainEnvironment FsWriter] Recursive directory copy from '{}' STUBBED.",
				SourcePath.display()
			);
			return Err(CommonError::NotImplemented { FeatureName:"Recursive directory copy".to_string() });
		}
		if OverwriteIfExists && fs::try_exists(TargetPath).await.unwrap_or(false) {
			debug!(
				"[MountainEnvironment FsWriter] Copy: Overwriting target by first deleting '{}'",
				TargetPath.display()
			);
			self.Delete(TargetPath, false, false).await?;
		}
		if let Some(TargetParentDir) = TargetPath.parent() {
			if !fs::try_exists(TargetParentDir).await.unwrap_or(false) {
				fs::create_dir_all(TargetParentDir).await.map_err(|IoError| {
					InternalUtils::MapIoErrorToCommonError(
						IoError,
						TargetParentDir.to_path_buf(),
						"mkdir_parent_for_copy",
					)
				})?;
			}
		}
		fs::copy(SourcePath, TargetPath)
			.await
			.map(|_| ())
			.map_err(|IoError| InternalUtils::MapIoErrorToCommonError(IoError, SourcePath.clone(), "copy"))?;
		Ok(())
	}
}

impl Requires<Arc<dyn FsWriter + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn FsWriter + Send + Sync> { Arc::new(self.clone()) }
}

// --- ConfigProvider & ConfigInspector Implementations ---
// ... (These will be similar to the original, using Handlers::Config, and
// adapted for PascalCase) ... (Located in the current
// `environment/config_provider.rs` file)
#[async_trait]
impl ConfigProvider for MountainEnvironment {
	async fn GetConfigurationValue(
		&self,
		SectionKeyOption:Option<String>,
		Overrides:IConfigurationOverrides,
	) -> Result<Value, CommonError> {
		InternalUtils::ConfigProviderImpl::GetConfigurationValue(self, SectionKeyOption, Overrides).await
	}

	async fn UpdateConfigurationValue(
		&self,
		KeyToUpdate:String,
		ValueToSet:Value,
		TargetScope:ConfigurationTarget,
		Overrides:IConfigurationOverrides,
		ScopeToLanguageOverride:Option<bool>,
	) -> Result<(), CommonError> {
		InternalUtils::ConfigProviderImpl::UpdateConfigurationValue(
			self,
			KeyToUpdate,
			ValueToSet,
			TargetScope,
			Overrides,
			ScopeToLanguageOverride,
		)
		.await
	}
}

#[async_trait]
impl ConfigInspector for MountainEnvironment {
	async fn InspectConfigurationValue(
		&self,
		Key:String,
		Overrides:IConfigurationOverrides,
	) -> Result<Option<InspectResultData>, CommonError> {
		InternalUtils::ConfigProviderImpl::InspectConfigurationValue(self, Key, Overrides).await
	}
}

impl Requires<Arc<dyn ConfigProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn ConfigProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn ConfigInspector + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn ConfigInspector + Send + Sync> { Arc::new(self.clone()) }
}

// --- DocumentProvider Implementation ---
// ... (Located in the current `environment/documents_provider.rs` file)
#[async_trait]
impl DocumentProvider for MountainEnvironment {
	async fn OpenDocument(
		&self,
		UriComponentsDto:Value,
		LanguageIdentifierOverrideOption:Option<String>,
		InitialContentOption:Option<String>,
	) -> Result<Url, CommonError> {
		InternalUtils::DocumentProviderImpl::OpenDocument(
			self,
			UriComponentsDto,
			LanguageIdentifierOverrideOption,
			InitialContentOption,
		)
		.await
	}

	async fn SaveDocument(&self, UriToSave:Url) -> Result<bool, CommonError> {
		InternalUtils::DocumentProviderImpl::SaveDocument(self, UriToSave).await
	}

	async fn SaveDocumentAs(
		&self,
		OriginalUri:Url,
		NewUriTargetOption:Option<Url>,
	) -> Result<Option<Url>, CommonError> {
		InternalUtils::DocumentProviderImpl::SaveDocumentAs(self, OriginalUri, NewUriTargetOption).await
	}

	async fn SaveAllDocuments(&self, IncludeUntitled:bool) -> Result<Vec<bool>, CommonError> {
		InternalUtils::DocumentProviderImpl::SaveAllDocuments(self, IncludeUntitled).await
	}

	async fn ApplyDocumentChanges(
		&self,
		UriToChange:Url,
		NewVersionIdentifier:i64,
		ChangesDtoCollectionValue:Value,
		IsDirtyAfterChange:bool,
		IsUndoingOperation:bool,
		IsRedoingOperation:bool,
	) -> Result<(), CommonError> {
		InternalUtils::DocumentProviderImpl::ApplyDocumentChanges(
			self,
			UriToChange,
			NewVersionIdentifier,
			ChangesDtoCollectionValue,
			IsDirtyAfterChange,
			IsUndoingOperation,
			IsRedoingOperation,
		)
		.await
	}
}

impl Requires<Arc<dyn DocumentProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn DocumentProvider + Send + Sync> { Arc::new(self.clone()) }
}

// --- StorageProvider Implementation ---
// ... (Located in the current `environment/storage_provider.rs` file)
#[async_trait]
impl StorageProvider for MountainEnvironment {
	async fn GetStorageValue(&self, IsGlobalScope:bool, Key:&str) -> Result<Option<Value>, CommonError> {
		InternalUtils::StorageProviderImpl::GetStorageValue(self, IsGlobalScope, Key).await
	}

	async fn UpdateStorageValue(
		&self,
		IsGlobalScope:bool,
		Key:String,
		ValueToSet:Option<Value>,
	) -> Result<(), CommonError> {
		InternalUtils::StorageProviderImpl::UpdateStorageValue(self, IsGlobalScope, Key, ValueToSet).await
	}
}
impl Requires<Arc<dyn StorageProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn StorageProvider + Send + Sync> { Arc::new(self.clone()) }
}

// --- SecretsProvider Implementation ---
// ... (Located in the current `environment/secrets_provider.rs` file)
#[async_trait]
impl SecretsProvider for MountainEnvironment {
	async fn GetSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<Option<String>, CommonError> {
		InternalUtils::SecretsProviderImpl::GetSecret(self, ExtensionIdentifier, Key).await
	}

	async fn StoreSecret(
		&self,
		ExtensionIdentifier:String,
		Key:String,
		ValueToStore:String,
	) -> Result<(), CommonError> {
		InternalUtils::SecretsProviderImpl::StoreSecret(self, ExtensionIdentifier, Key, ValueToStore).await
	}

	async fn DeleteSecret(&self, ExtensionIdentifier:String, Key:String) -> Result<(), CommonError> {
		InternalUtils::SecretsProviderImpl::DeleteSecret(self, ExtensionIdentifier, Key).await
	}
}
impl Requires<Arc<dyn SecretsProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn SecretsProvider + Send + Sync> { Arc::new(self.clone()) }
}

// --- OutputChannelManager Implementation ---
// ... (Located in the current `environment/output_provider.rs` file)
#[async_trait]
impl OutputChannelManager for MountainEnvironment {
	async fn RegisterChannel(&self, Name:String, LanguageIdentifier:Option<String>) -> Result<String, CommonError> {
		InternalUtils::OutputProviderImpl::RegisterChannel(self, Name, LanguageIdentifier).await
	}

	async fn Append(&self, ChannelIdentifier:String, Value:String) -> Result<(), CommonError> {
		InternalUtils::OutputProviderImpl::Append(self, ChannelIdentifier, Value).await
	}

	async fn Replace(&self, ChannelIdentifier:String, Value:String) -> Result<(), CommonError> {
		InternalUtils::OutputProviderImpl::Replace(self, ChannelIdentifier, Value).await
	}

	async fn Clear(&self, ChannelIdentifier:String) -> Result<(), CommonError> {
		InternalUtils::OutputProviderImpl::Clear(self, ChannelIdentifier).await
	}

	async fn Reveal(&self, ChannelIdentifier:String, PreserveFocus:bool) -> Result<(), CommonError> {
		InternalUtils::OutputProviderImpl::Reveal(self, ChannelIdentifier, PreserveFocus).await
	}

	async fn Close(&self, ChannelIdentifier:String) -> Result<(), CommonError> {
		InternalUtils::OutputProviderImpl::Close(self, ChannelIdentifier).await
	}

	async fn Dispose(&self, ChannelIdentifier:String) -> Result<(), CommonError> {
		InternalUtils::OutputProviderImpl::Dispose(self, ChannelIdentifier).await
	}
}
impl Requires<Arc<dyn OutputChannelManager + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn OutputChannelManager + Send + Sync> { Arc::new(self.clone()) }
}

// --- DiagnosticsManager Implementation ---
// ... (Located in the current `environment/diagnostics_provider.rs` file)
#[async_trait]
impl DiagnosticsManager for MountainEnvironment {
	async fn SetDiagnostics(&self, Owner:String, EntriesDtoValue:Value) -> Result<(), CommonError> {
		InternalUtils::DiagnosticsProviderImpl::SetDiagnostics(self, Owner, EntriesDtoValue).await
	}

	async fn ClearDiagnostics(&self, Owner:String) -> Result<(), CommonError> {
		InternalUtils::DiagnosticsProviderImpl::ClearDiagnostics(self, Owner).await
	}

	async fn GetAllDiagnostics(&self, ResourceUriFilterOption:Option<Value>) -> Result<Value, CommonError> {
		InternalUtils::DiagnosticsProviderImpl::GetAllDiagnostics(self, ResourceUriFilterOption).await
	}
}
impl Requires<Arc<dyn DiagnosticsManager + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn DiagnosticsManager + Send + Sync> { Arc::new(self.clone()) }
}

// --- CommandExecutor Implementation ---
// ... (Located in the current `environment/commands_provider.rs` file)
#[async_trait]
impl CommandExecutor for MountainEnvironment {
	async fn ExecuteCommand(&self, CommandIdentifier:String, ArgumentValue:Value) -> Result<Value, CommonError> {
		InternalUtils::CommandsProviderImpl::ExecuteCommand(self, CommandIdentifier, ArgumentValue).await
	}

	async fn RegisterCommand(&self, SidecarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError> {
		InternalUtils::CommandsProviderImpl::RegisterCommand(self, SidecarIdentifier, CommandIdentifier).await
	}

	async fn UnregisterCommand(&self, SidecarIdentifier:String, CommandIdentifier:String) -> Result<(), CommonError> {
		InternalUtils::CommandsProviderImpl::UnregisterCommand(self, SidecarIdentifier, CommandIdentifier).await
	}

	async fn GetAllCommands(&self) -> Result<Vec<String>, CommonError> {
		InternalUtils::CommandsProviderImpl::GetAllCommands(self).await
	}
}
impl Requires<Arc<dyn CommandExecutor + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn CommandExecutor + Send + Sync> { Arc::new(self.clone()) }
}

// --- WorkspaceProvider & WorkspaceEditApplier Implementations ---
// ... (Located in the current `environment/workspace_provider.rs` file)
#[async_trait]
impl WorkspaceProvider for MountainEnvironment {
	async fn GetWorkspaceFoldersInfo(&self) -> Result<Vec<(Url, String, usize)>, CommonError> {
		InternalUtils::WorkspaceProviderImpl::GetWorkspaceFoldersInfo(self).await
	}

	async fn GetWorkspaceFolderInfo(&self, UriToMatch:Url) -> Result<Option<(Url, String, usize)>, CommonError> {
		InternalUtils::WorkspaceProviderImpl::GetWorkspaceFolderInfo(self, UriToMatch).await
	}

	async fn GetWorkspaceName(&self) -> Result<Option<String>, CommonError> {
		InternalUtils::WorkspaceProviderImpl::GetWorkspaceName(self).await
	}

	async fn GetWorkspaceConfigurationPath(&self) -> Result<Option<PathBuf>, CommonError> {
		InternalUtils::WorkspaceProviderImpl::GetWorkspaceConfigurationPath(self).await
	}

	async fn IsWorkspaceTrusted(&self) -> Result<bool, CommonError> {
		InternalUtils::WorkspaceProviderImpl::IsWorkspaceTrusted(self).await
	}

	async fn RequestWorkspaceTrust(&self, Options:Option<Value>) -> Result<bool, CommonError> {
		InternalUtils::WorkspaceProviderImpl::RequestWorkspaceTrust(self, Options).await
	}

	async fn FindFilesInWorkspace(
		&self,
		IncludePatternDto:Value,
		ExcludePatternDto:Option<Value>,
		MaxResults:Option<usize>,
		UseIgnoreFiles:bool,
		FollowSymlinks:bool,
	) -> Result<Vec<Url>, CommonError> {
		InternalUtils::WorkspaceProviderImpl::FindFilesInWorkspace(
			self,
			IncludePatternDto,
			ExcludePatternDto,
			MaxResults,
			UseIgnoreFiles,
			FollowSymlinks,
		)
		.await
	}

	async fn OpenFile(&self, Path:PathBuf) -> Result<(), CommonError> {
		InternalUtils::WorkspaceProviderImpl::OpenFile(self, Path).await
	}
}

#[async_trait]
impl WorkspaceEditApplier for MountainEnvironment {
	async fn ApplyWorkspaceEdit(&self, EditDto:WorkspaceEditDto) -> Result<bool, CommonError> {
		InternalUtils::WorkspaceProviderImpl::ApplyWorkspaceEdit(self, EditDto).await
	}
}
impl Requires<Arc<dyn WorkspaceProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn WorkspaceProvider + Send + Sync> { Arc::new(self.clone()) }
}
impl Requires<Arc<dyn WorkspaceEditApplier + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn WorkspaceEditApplier + Send + Sync> { Arc::new(self.clone()) }
}

// --- UiProvider Implementation ---
// ... (Located in the current `environment/ui_provider.rs` file)
#[async_trait]
impl UiProvider for MountainEnvironment {
	async fn ShowMessage(
		&self,
		Severity:MessageSeverity,
		MessageText:String,
		OptionsJsonValueOption:Option<Value>,
	) -> Result<Option<String>, CommonError> {
		InternalUtils::UiProviderImpl::ShowMessage(self, Severity, MessageText, OptionsJsonValueOption).await
	}

	async fn ShowOpenDialog(&self, Options:Option<OpenDialogOptions>) -> Result<Option<Vec<PathBuf>>, CommonError> {
		InternalUtils::UiProviderImpl::ShowOpenDialog(self, Options).await
	}

	async fn ShowSaveDialog(&self, Options:Option<SaveDialogOptions>) -> Result<Option<PathBuf>, CommonError> {
		InternalUtils::UiProviderImpl::ShowSaveDialog(self, Options).await
	}

	async fn ShowQuickPick(
		&self,
		Items:Vec<QuickPickItem>,
		Options:Option<QuickPickOptions>,
	) -> Result<Option<Vec<String>>, CommonError> {
		InternalUtils::UiProviderImpl::ShowQuickPick(self, Items, Options).await
	}

	async fn ShowInputBox(&self, Options:Option<InputBoxOptions>) -> Result<Option<String>, CommonError> {
		InternalUtils::UiProviderImpl::ShowInputBox(self, Options).await
	}
}
impl Requires<Arc<dyn UiProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn UiProvider + Send + Sync> { Arc::new(self.clone()) }
}

// --- IpcProvider Implementation ---
// ... (Located in the current `environment/ipc_provider.rs` file)
#[async_trait]
impl IpcProvider for MountainEnvironment {
	async fn SendNotificationToSidecar(
		&self,
		SidecarIdentifier:String,
		Method:String,
		Parameters:Value,
	) -> Result<(), CommonError> {
		InternalUtils::IpcProviderImpl::SendNotificationToSidecar(self, SidecarIdentifier, Method, Parameters).await
	}

	async fn SendRequestToSidecar(
		&self,
		SidecarIdentifier:String,
		Method:String,
		Parameters:Value,
		TimeoutMilliseconds:u64,
	) -> Result<Value, CommonError> {
		InternalUtils::IpcProviderImpl::SendRequestToSidecar(
			self,
			SidecarIdentifier,
			Method,
			Parameters,
			TimeoutMilliseconds,
		)
		.await
	}
}
impl Requires<Arc<dyn IpcProvider + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn IpcProvider + Send + Sync> { Arc::new(self.clone()) }
}

// --- LanguageFeatureProviderRegistry Implementation ---
// ... (Located in the current `environment/language_features_provider.rs` file)
// ... This will be a large one, similar to the original file.
// ... It will use the macros for `ProvideFeature!` and `ResolveFeatureItem!`
#[async_trait]
impl LanguageFeatureProviderRegistry for MountainEnvironment {
	async fn GetProvidersForDocument(
		&self,
		DocumentUri:Url,
		LanguageIdentifier:String,
		ProviderTypeToMatch:CommonProviderType,
	) -> Result<Vec<ProviderDescription>, CommonError> {
		InternalUtils::LanguageFeaturesProviderImpl::GetProvidersForDocument(
			self,
			DocumentUri,
			LanguageIdentifier,
			ProviderTypeToMatch,
		)
		.await
	}

	// ... (all other provide_ and resolve_ methods, adapted to PascalCase and using
	// the macros or direct logic) ... I will fill these in based on the previous
	// `language_features_provider.rs` content. ... For brevity here, I'm omitting
	// the full list, but they would follow the pattern.

	async fn ProvideHover(
		&self,
		DocumentUri:Url,
		LanguageIdentifier:String,
		PositionDtoInput:PositionDto,
	) -> Result<Option<HoverResultDto>, CommonError> {
		InternalUtils::LanguageFeaturesProviderImpl::ProvideHover(
			self,
			DocumentUri,
			LanguageIdentifier,
			PositionDtoInput,
		)
		.await
	}

	async fn ProvideCompletions(
		&self,
		DocumentUri:Url,
		LanguageIdentifier:String,
		PositionDtoInput:PositionDto,
		ContextDtoInput:CompletionContextDto,
		CancellationTokenIdentifierValue:Option<Value>,
	) -> Result<Option<SuggestResultDto>, CommonError> {
		InternalUtils::LanguageFeaturesProviderImpl::ProvideCompletions(
			self,
			DocumentUri,
			LanguageIdentifier,
			PositionDtoInput,
			ContextDtoInput,
			CancellationTokenIdentifierValue,
		)
		.await
	}

	async fn ResolveCompletionItemForList(
		&self,
		ListCacheIdentifier:u32,
		ItemToResolveDto:Value,
		CancellationTokenIdentifierValue:Option<Value>,
	) -> Result<Option<Value>, CommonError> {
		InternalUtils::LanguageFeaturesProviderImpl::ResolveCompletionItemForList(
			self,
			ListCacheIdentifier,
			ItemToResolveDto,
			CancellationTokenIdentifierValue,
		)
		.await
	}
	// ... other methods ...
}
impl Requires<Arc<dyn LanguageFeatureProviderRegistry + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn LanguageFeatureProviderRegistry + Send + Sync> { Arc::new(self.clone()) }
}

// Placeholder for nested provider implementations within InternalUtils
// This is where the content from individual provider files like
// commands_provider.rs, config_provider.rs etc. would be refactored into.
// For example:
// mod InternalUtils {
//   pub mod ConfigProviderImpl { async fn GetConfigurationValue(...) { ... } }
//   pub mod DocumentProviderImpl { async fn OpenDocument(...) { ... } }
//   ... etc.
// }
// Then the main impls above would call these, e.g.:
// InternalUtils::ConfigProviderImpl::GetConfigurationValue(self, ...).await
