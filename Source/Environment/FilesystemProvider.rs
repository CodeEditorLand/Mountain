
// Implements the `FsReader` and `FsWriter` traits for the
// `MountainEnvironment`. This file connects abstract filesystem effects to the
// concrete logic using `tokio::fs`.

#![allow(non_snake_case, non_camel_case_types)]

use std::{path::PathBuf, sync::Arc};

use Common::{
	Environment::Requires,
	Errors::CommonError,
	FsEffect::{FileSystemStat, FileType as CommonFileType, FsReader, FsWriter},
};
use async_trait::async_trait;
use log::{debug, error, info, trace, warn};
use tokio::fs;

use crate::Environment::{
	MountainEnvironment,
	Utils::{self, IsPathAllowedForFilesystemAccess, MapIoErrorToCommonError},
};

#[async_trait]
impl FsReader for MountainEnvironment {
	/// Reads the entire contents of a file into a bytes vector after ensuring
	/// path is allowed.
	async fn ReadFile(&self, Path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		trace!("[Environment FilesystemProvider] ReadFile: {}", Path.display());
		fs::read(Path)
			.await
			.map_err(|IoError| MapIoErrorToCommonError(IoError, Path.clone(), "read"))
	}

	/// Retrieves metadata for a file or directory after ensuring path is
	/// allowed.
	async fn StatFile(&self, Path:&PathBuf) -> Result<FileSystemStat, CommonError> {
		IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		trace!("[Environment FilesystemProvider] StatFile: {}", Path.display());
		match fs::metadata(Path).await {
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

				let GetMilliTimestamp = |SystemTimeResult:Result<std::time::SystemTime, _>| -> u64 {
					SystemTimeResult
						.ok()
						.and_then(|Time| Time.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
						.map_or(0, |Duration| Duration.as_millis() as u64)
				};

				Ok(FileSystemStat {
					FileType:FileTypeFlags,
					CreationTime:GetMilliTimestamp(Metadata.created()),
					ModificationTime:GetMilliTimestamp(Metadata.modified()),
					Size:Metadata.len(),
					Permissions:None,
				})
			},
			Err(IoError) => Err(MapIoErrorToCommonError(IoError, Path.clone(), "stat")),
		}
	}

	/// Reads the contents of a directory after ensuring path is allowed.
	async fn ReadDirectory(&self, Path:&PathBuf) -> Result<Vec<(String, CommonFileType)>, CommonError> {
		IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		debug!("[Environment FilesystemProvider] ReadDirectory: {}", Path.display());
		let mut EntriesVector:Vec<(String, CommonFileType)> = Vec::new();
		let mut DirectoryEntriesStream = fs::read_dir(Path)
			.await
			.map_err(|IoError| MapIoErrorToCommonError(IoError, Path.clone(), "readdir"))?;

		while let Some(DirectoryEntryResult) = DirectoryEntriesStream
			.next_entry()
			.await
			.map_err(|IoError| MapIoErrorToCommonError(IoError, Path.clone(), "readdir_next_entry"))?
		{
			let FileNameOsString = DirectoryEntryResult.file_name();
			let FileNameString = FileNameOsString.to_string_lossy().into_owned();
			match DirectoryEntryResult.file_type().await {
				Ok(TokioFileType) => {
					let CommonFileTypeInstance = if TokioFileType.is_dir() {
						CommonFileType::Directory
					} else if TokioFileType.is_file() {
						CommonFileType::File
					} else if TokioFileType.is_symlink() {
						CommonFileType::SymbolicLink
					} else {
						CommonFileType::Unknown
					};
					EntriesVector.push((FileNameString, CommonFileTypeInstance));
				},
				Err(FileTypeRetrievalError) => {
					warn!(
						"[Environment FilesystemProvider] Failed to get type for '{}' in '{}': {}. Marking as Unknown.",
						FileNameString,
						Path.display(),
						FileTypeRetrievalError
					);
					EntriesVector.push((FileNameString, CommonFileType::Unknown));
				},
			}
		}
		Ok(EntriesVector)
	}
}

#[async_trait]
impl FsWriter for MountainEnvironment {
	/// Writes content to a file after ensuring path is allowed.
	async fn WriteFile(
		&self,
		Path:&PathBuf,
		ContentBytes:Vec<u8>,
		CreateIfNotExists:bool,
		OverwriteIfExists:bool,
	) -> Result<(), CommonError> {
		IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		info!(
			"[Environment FilesystemProvider] WriteFile: Path='{}', Length={}, Create={}, Overwrite={}",
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

		if let Some(ParentDirectoryPath) = Path.parent() {
			if !fs::try_exists(ParentDirectoryPath).await.unwrap_or(false) {
				if CreateIfNotExists {
					fs::create_dir_all(ParentDirectoryPath).await.map_err(|IoError| {
						MapIoErrorToCommonError(IoError, ParentDirectoryPath.to_path_buf(), "mkdir_parent_for_write")
					})?;
				} else {
					return Err(CommonError::FsNotFound(ParentDirectoryPath.to_path_buf()));
				}
			}
		}
		fs::write(Path, &ContentBytes)
			.await
			.map_err(|IoError| MapIoErrorToCommonError(IoError, Path.clone(), "write"))
	}

	/// Creates a directory after ensuring path is allowed.
	async fn CreateDirectory(&self, Path:&PathBuf, RecursiveCreate:bool) -> Result<(), CommonError> {
		IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		info!(
			"[Environment FilesystemProvider] CreateDirectory: Path='{}', Recursive={}",
			Path.display(),
			RecursiveCreate
		);
		if RecursiveCreate {
			fs::create_dir_all(Path)
				.await
				.map_err(|IoError| MapIoErrorToCommonError(IoError, Path.clone(), "mkdir_all"))
		} else {
			fs::create_dir(Path)
				.await
				.map_err(|IoError| MapIoErrorToCommonError(IoError, Path.clone(), "mkdir"))
		}
	}

	/// Deletes a file or directory after ensuring path is allowed.
	async fn Delete(&self, Path:&PathBuf, RecursiveDelete:bool, UseOsTrash:bool) -> Result<(), CommonError> {
		IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		info!(
			"[Environment FilesystemProvider] Delete: Path='{}', Recursive={}, UseTrash={}",
			Path.display(),
			RecursiveDelete,
			UseOsTrash
		);
		if UseOsTrash {
			warn!(
				"[Environment FilesystemProvider] `UseOsTrash=true` is requested but not implemented; performing \
				 permanent delete."
			);
		}

		match fs::metadata(Path).await {
			Ok(Metadata) => {
				let DeleteOperation = if Metadata.is_dir() {
					if RecursiveDelete {
						fs::remove_dir_all(Path).await
					} else {
						fs::remove_dir(Path).await
					}
				} else {
					fs::remove_file(Path).await
				};
				DeleteOperation.map_err(|IoError| MapIoErrorToCommonError(IoError, Path.clone(), "delete"))
			},
			Err(Error) if Error.kind() == std::io::ErrorKind::NotFound => {
				debug!(
					"[Environment FilesystemProvider] Path '{}' not found for deletion (idempotent success).",
					Path.display()
				);
				Ok(())
			},
			Err(IoError) => Err(MapIoErrorToCommonError(IoError, Path.clone(), "delete_stat_check")),
		}
	}

	/// Renames a file or directory after ensuring paths are allowed.
	async fn Rename(
		&self,
		SourcePath:&PathBuf,
		TargetPath:&PathBuf,
		OverwriteIfExists:bool,
	) -> Result<(), CommonError> {
		IsPathAllowedForFilesystemAccess(&self.AppHandle, SourcePath).await?;
		IsPathAllowedForFilesystemAccess(&self.AppHandle, TargetPath).await?;
		info!(
			"[Environment FilesystemProvider] Rename: From='{}', To='{}', Overwrite={}",
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

		if let Some(TargetParentDir) = TargetPath.parent() {
			if !fs::try_exists(TargetParentDir).await.unwrap_or(false) {
				fs::create_dir_all(TargetParentDir).await.map_err(|IoError| {
					MapIoErrorToCommonError(IoError, TargetParentDir.to_path_buf(), "mkdir_parent_for_rename")
				})?;
			}
		}
		fs::rename(SourcePath, TargetPath)
			.await
			.map_err(|IoError| MapIoErrorToCommonError(IoError, SourcePath.clone(), "rename"))
	}

	/// Copies a file after ensuring paths are allowed.
	async fn Copy(&self, SourcePath:&PathBuf, TargetPath:&PathBuf, OverwriteIfExists:bool) -> Result<(), CommonError> {
		IsPathAllowedForFilesystemAccess(&self.AppHandle, SourcePath).await?;
		IsPathAllowedForFilesystemAccess(&self.AppHandle, TargetPath).await?;
		info!(
			"[Environment FilesystemProvider] Copy: From='{}', To='{}', Overwrite={}",
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

		let SourceMetadata = fs::metadata(SourcePath)
			.await
			.map_err(|IoError| MapIoErrorToCommonError(IoError, SourcePath.clone(), "copy_source_stat"))?;
		if SourceMetadata.is_dir() {
			error!(
				"[Environment FilesystemProvider] Recursive directory copy from '{}' is not yet implemented.",
				SourcePath.display()
			);
			return Err(CommonError::NotImplemented { FeatureName:"Recursive directory copy".to_string() });
		}

		if let Some(TargetParentDir) = TargetPath.parent() {
			if !fs::try_exists(TargetParentDir).await.unwrap_or(false) {
				fs::create_dir_all(TargetParentDir).await.map_err(|IoError| {
					MapIoErrorToCommonError(IoError, TargetParentDir.to_path_buf(), "mkdir_parent_for_copy")
				})?;
			}
		}
		fs::copy(SourcePath, TargetPath)
			.await
			.map(|_| ())
			.map_err(|IoError| MapIoErrorToCommonError(IoError, SourcePath.clone(), "copy"))
	}

	/// Creates an empty file after ensuring path is allowed.
	async fn CreateFile(&self, Path:&PathBuf) -> Result<(), CommonError> {
		IsPathAllowedForFilesystemAccess(&self.AppHandle, Path).await?;
		info!("[Environment FilesystemProvider] CreateFile: {}", Path.display());
		if fs::try_exists(Path).await.unwrap_or(false) {
			return Err(CommonError::FsFileExists(Path.clone()));
		}
		if let Some(ParentDir) = Path.parent() {
			if !fs::try_exists(ParentDir).await.unwrap_or(false) {
				fs::create_dir_all(ParentDir).await.map_err(|IoError| {
					MapIoErrorToCommonError(IoError, ParentDir.to_path_buf(), "mkdir_parent_for_create_file")
				})?;
			}
		}
		fs::File::create(Path)
			.await
			.map(|_| ())
			.map_err(|IoError| MapIoErrorToCommonError(IoError, Path.clone(), "create_file"))
	}
}

impl Requires<Arc<dyn FsWriter + Send + Sync>> for MountainEnvironment {
	fn require(&self) -> Arc<dyn FsWriter + Send + Sync> { Arc::new(self.clone()) }
}
