use std::path::{Path, PathBuf};

use Common::{
	error::CommonError,
	fs::dto::{FileSystemStatDto, FileTypeDto},
};
use tauri::{ApplicationHandle, Wry};
use tokio::fs;

// @module FsLogic
// @description Contains the core, detailed logic for all native filesystem
// operations, using `tokio::fs` for asynchronous I/O.
use crate::environment::Utils;

// Logic to read the entire contents of a file into a byte vector.
pub async fn ReadFileLogic(ApplicationHandle:&ApplicationHandle<Wry>, Path:&PathBuf) -> Result<Vec<u8>, CommonError> {
	Utils::IsPathAllowedForFilesystemAccess(ApplicationHandle, Path).await?;
	fs::read(Path)
		.await
		.map_err(|e| Utils::MapIoErrorToCommonError(e, Path.clone(), "ReadFile"))
}

// Logic to retrieve metadata for a file or directory.
pub async fn StatFileLogic(ApplicationHandle:&ApplicationHandle<Wry>, Path:&PathBuf) -> Result<FileSystemStatDto, CommonError> {
	Utils::IsPathAllowedForFilesystemAccess(ApplicationHandle, Path).await?;
	let Metadata = fs::metadata(Path)
		.await
		.map_err(|e| Utils::MapIoErrorToCommonError(e, Path.clone(), "StatFile"))?;

	let mut FileType = 0_u8;
	if Metadata.is_file() {
		FileType |= FileTypeDto::File as u8;
	}
	if Metadata.is_dir() {
		FileType |= FileTypeDto::Directory as u8;
	}
	if Metadata.is_symlink() {
		FileType |= FileTypeDto::SymbolicLink as u8;
	}

	let GetMilliTimestamp = |SystemTimeResult:Result<std::time::SystemTime, _>| -> u64 {
		SystemTimeResult
			.ok()
			.and_then(|Time| Time.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
			.map_or(0, |Duration| Duration.as_millis() as u64)
	};

	Ok(FileSystemStatDto {
		FileType,
		CreationTime:GetMilliTimestamp(Metadata.created()),
		ModificationTime:GetMilliTimestamp(Metadata.modified()),
		Size:Metadata.len(),
		Permissions:None,
	})
}

// Logic to read the contents of a directory.
pub async fn ReadDirectoryLogic(
	ApplicationHandle:&ApplicationHandle<Wry>,
	Path:&PathBuf,
) -> Result<Vec<(String, FileTypeDto)>, CommonError> {
	Utils::IsPathAllowedForFilesystemAccess(ApplicationHandle, Path).await?;
	let mut Entries = Vec::new();
	let mut ReadDir = fs::read_dir(Path)
		.await
		.map_err(|e| Utils::MapIoErrorToCommonError(e, Path.clone(), "ReadDirectory"))?;

	while let Some(EntryResult) = ReadDir
		.next_entry()
		.await
		.map_err(|e| Utils::MapIoErrorToCommonError(e, Path.clone(), "ReadDirectory.NextEntry"))?
	{
		let FileName = EntryResult.file_name().to_string_lossy().into_owned();
		let FileType = match EntryResult.file_type().await {
			Ok(Type) => {
				if Type.is_dir() {
					FileTypeDto::Directory
				} else if Type.is_file() {
					FileTypeDto::File
				} else {
					FileTypeDto::Unknown
				}
			},
			Err(_) => FileTypeDto::Unknown,
		};
		Entries.push((FileName, FileType));
	}
	Ok(Entries)
}

// Logic to write content to a file, with options for creation and overwriting.
pub async fn WriteFileLogic(
	ApplicationHandle:&ApplicationHandle<Wry>,
	Path:&PathBuf,
	Content:Vec<u8>,
	Create:bool,
	Overwrite:bool,
) -> Result<(), CommonError> {
	Utils::IsPathAllowedForFilesystemAccess(ApplicationHandle, Path).await?;
	let PathExists = fs::try_exists(Path).await.unwrap_or(false);

	if PathExists && !Overwrite {
		return Err(CommonError::FsFileExists(Path.clone()));
	}
	if !PathExists && !Create {
		return Err(CommonError::FsNotFound(Path.clone()));
	}

	if let Some(ParentDir) = Path.parent() {
		if !fs::try_exists(ParentDir).await.unwrap_or(false) {
			fs::create_dir_all(ParentDir)
				.await
				.map_err(|e| Utils::MapIoErrorToCommonError(e, ParentDir.to_path_buf(), "WriteFile.CreateParent"))?;
		}
	}

	fs::write(Path, &Content)
		.await
		.map_err(|e| Utils::MapIoErrorToCommonError(e, Path.clone(), "WriteFile"))
}

// Logic to create a directory, with an option for recursive creation.
pub async fn CreateDirectoryLogic(ApplicationHandle:&ApplicationHandle<Wry>, Path:&PathBuf, Recursive:bool) -> Result<(), CommonError> {
	Utils::IsPathAllowedForFilesystemAccess(ApplicationHandle, Path).await?;
	let operation = if Recursive {
		fs::create_dir_all(Path).await
	} else {
		fs::create_dir(Path).await
	};
	operation.map_err(|e| Utils::MapIoErrorToCommonError(e, Path.clone(), "CreateDirectory"))
}

// Logic to delete a file or directory. The operation is idempotent.
pub async fn DeleteLogic(
	ApplicationHandle:&ApplicationHandle<Wry>,
	Path:&PathBuf,
	Recursive:bool,
	_UseTrash:bool,
) -> Result<(), CommonError> {
	Utils::IsPathAllowedForFilesystemAccess(ApplicationHandle, Path).await?;
	match fs::metadata(Path).await {
		Ok(Metadata) => {
			let operation = if Metadata.is_dir() {
				if Recursive {
					fs::remove_dir_all(Path).await
				} else {
					fs::remove_dir(Path).await
				}
			} else {
				fs::remove_file(Path).await
			};
			operation.map_err(|e| Utils::MapIoErrorToCommonError(e, Path.clone(), "Delete"))
		},
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Success if it's already gone.
		Err(e) => Err(Utils::MapIoErrorToCommonError(e, Path.clone(), "Delete.Stat")),
	}
}

// Logic to rename (move) a file or directory.
pub async fn RenameLogic(
	ApplicationHandle:&ApplicationHandle<Wry>,
	Source:&PathBuf,
	Target:&PathBuf,
	Overwrite:bool,
) -> Result<(), CommonError> {
	Utils::IsPathAllowedForFilesystemAccess(ApplicationHandle, Source).await?;
	Utils::IsPathAllowedForFilesystemAccess(ApplicationHandle, Target).await?;
	if !Overwrite && fs::try_exists(Target).await.unwrap_or(false) {
		return Err(CommonError::FsFileExists(Target.clone()));
	}
	fs::rename(Source, Target)
		.await
		.map_err(|e| Utils::MapIoErrorToCommonError(e, Source.clone(), "Rename"))
}

// Logic to copy a file. Does not support recursive directory copy.
pub async fn CopyLogic(
	ApplicationHandle:&ApplicationHandle<Wry>,
	Source:&PathBuf,
	Target:&PathBuf,
	Overwrite:bool,
) -> Result<(), CommonError> {
	Utils::IsPathAllowedForFilesystemAccess(ApplicationHandle, Source).await?;
	Utils::IsPathAllowedForFilesystemAccess(ApplicationHandle, Target).await?;
	let SourceMetadata = StatFileLogic(ApplicationHandle, Source).await?;
	if SourceMetadata.FileType == FileTypeDto::Directory as u8 {
		return Err(CommonError::NotImplemented { FeatureName:"Recursive directory copy".to_string() });
	}
	if !Overwrite && fs::try_exists(Target).await.unwrap_or(false) {
		return Err(CommonError::FsFileExists(Target.clone()));
	}
	fs::copy(Source, Target)
		.await
		.map(|_| ())
		.map_err(|e| Utils::MapIoErrorToCommonError(e, Source.clone(), "Copy"))
}

// Logic to create an empty file.
pub async fn CreateFileLogic(ApplicationHandle:&ApplicationHandle<Wry>, Path:&PathBuf) -> Result<(), CommonError> {
	WriteFileLogic(ApplicationHandle, Path, vec![], true, false).await
}
