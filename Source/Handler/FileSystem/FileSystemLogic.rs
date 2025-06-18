// @module FileSystemLogic
// @description Contains the core, detailed logic for all native filesystem
// operations, using `tokio::fs` for asynchronous I/O.

use std::path::{Path, PathBuf};

use Common::{
	error::CommonError,
	fs::DTO::{FileSystemStatDTO, FileTypeDTO},
};
use tauri::{AppHandle, Wry};
use tokio::fs;

use crate::Environment::Utility as EnvUtils;

// Logic to read the entire contents of a file into a byte vector.
pub async fn ReadFileLogic(app_handle:&AppHandle<Wry>, path:&PathBuf) -> Result<Vec<u8>, CommonError> {
	EnvUtils::IsPathAllowedForFilesystemAccess(app_handle, path).await?;
	fs::read(path)
		.await
		.map_err(|e| EnvUtils::MapIoErrorToCommonError(e, path.clone(), "ReadFile"))
}

// Logic to retrieve metadata for a file or directory.
pub async fn StatFileLogic(app_handle:&AppHandle<Wry>, path:&PathBuf) -> Result<FileSystemStatDTO, CommonError> {
	EnvUtils::IsPathAllowedForFilesystemAccess(app_handle, path).await?;
	let metadata = fs::metadata(path)
		.await
		.map_err(|e| EnvUtils::MapIoErrorToCommonError(e, path.clone(), "StatFile"))?;

	let mut file_type = 0_u8;
	if metadata.is_file() {
		file_type |= FileTypeDTO::File as u8;
	}
	if metadata.is_dir() {
		file_type |= FileTypeDTO::Directory as u8;
	}
	// Note: is_symlink() is not mutually exclusive with is_file() or is_dir()
	if metadata.file_type().is_symlink() {
		file_type |= FileTypeDTO::SymbolicLink as u8;
	}

	let get_milli_timestamp = |system_time_result:Result<std::time::SystemTime, _>| -> u64 {
		system_time_result
			.ok()
			.and_then(|time| time.duration_since(std::time::SystemTime::UNIX_EPOCH).ok())
			.map_or(0, |duration| duration.as_millis() as u64)
	};

	Ok(FileSystemStatDTO {
		FileType:file_type,
		CreationTime:get_milli_timestamp(metadata.created()),
		ModificationTime:get_milli_timestamp(metadata.modified()),
		Size:metadata.len(),
		Permissions:None,
	})
}

// Logic to read the contents of a directory.
pub async fn ReadDirectoryLogic(
	app_handle:&AppHandle<Wry>,
	path:&PathBuf,
) -> Result<Vec<(String, FileTypeDTO)>, CommonError> {
	EnvUtils::IsPathAllowedForFilesystemAccess(app_handle, path).await?;
	let mut entries = Vec::new();
	let mut read_dir = fs::read_dir(path)
		.await
		.map_err(|e| EnvUtils::MapIoErrorToCommonError(e, path.clone(), "ReadDirectory"))?;

	while let Some(entry_result) = read_dir
		.next_entry()
		.await
		.map_err(|e| EnvUtils::MapIoErrorToCommonError(e, path.clone(), "ReadDirectory.NextEntry"))?
	{
		let file_name = entry_result.file_name().to_string_lossy().into_owned();
		let file_type = match entry_result.file_type().await {
			Ok(ft) => {
				if ft.is_dir() {
					FileTypeDTO::Directory
				} else if ft.is_file() {
					FileTypeDTO::File
				} else {
					FileTypeDTO::Unknown
				}
			},
			Err(_) => FileTypeDTO::Unknown,
		};
		entries.push((file_name, file_type));
	}
	Ok(entries)
}

// Logic to write content to a file, with options for creation and overwriting.
pub async fn WriteFileLogic(
	app_handle:&AppHandle<Wry>,
	path:&PathBuf,
	content:Vec<u8>,
	create:bool,
	overwrite:bool,
) -> Result<(), CommonError> {
	EnvUtils::IsPathAllowedForFilesystemAccess(app_handle, path).await?;
	let path_exists = fs::try_exists(path).await.unwrap_or(false);

	if path_exists && !overwrite {
		return Err(CommonError::FileSystemFileExists(path.clone()));
	}
	if !path_exists && !create {
		return Err(CommonError::FileSystemNotFound(path.clone()));
	}

	if let Some(parent_dir) = path.parent() {
		if !fs::try_exists(parent_dir).await.unwrap_or(false) {
			fs::create_dir_all(parent_dir).await.map_err(|e| {
				EnvUtils::MapIoErrorToCommonError(e, parent_dir.to_path_buf(), "WriteFile.CreateParent")
			})?;
		}
	}

	fs::write(path, &content)
		.await
		.map_err(|e| EnvUtils::MapIoErrorToCommonError(e, path.clone(), "WriteFile"))
}

// Logic to create a directory, with an option for recursive creation.
pub async fn CreateDirectoryLogic(
	app_handle:&AppHandle<Wry>,
	path:&PathBuf,
	recursive:bool,
) -> Result<(), CommonError> {
	EnvUtils::IsPathAllowedForFilesystemAccess(app_handle, path).await?;
	let operation = if recursive {
		fs::create_dir_all(path).await
	} else {
		fs::create_dir(path).await
	};
	operation.map_err(|e| EnvUtils::MapIoErrorToCommonError(e, path.clone(), "CreateDirectory"))
}

// Logic to delete a file or directory. The operation is idempotent.
pub async fn DeleteLogic(
	app_handle:&AppHandle<Wry>,
	path:&PathBuf,
	recursive:bool,
	_use_trash:bool, // `trash` crate could be used here.
) -> Result<(), CommonError> {
	EnvUtils::IsPathAllowedForFilesystemAccess(app_handle, path).await?;
	match fs::metadata(path).await {
		Ok(metadata) => {
			let operation = if metadata.is_dir() {
				if recursive {
					fs::remove_dir_all(path).await
				} else {
					fs::remove_dir(path).await
				}
			} else {
				fs::remove_file(path).await
			};
			operation.map_err(|e| EnvUtils::MapIoErrorToCommonError(e, path.clone(), "Delete"))
		},
		Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()), // Success if it's already gone.
		Err(e) => Err(EnvUtils::MapIoErrorToCommonError(e, path.clone(), "Delete.Stat")),
	}
}

// Logic to rename (move) a file or directory.
pub async fn RenameLogic(
	app_handle:&AppHandle<Wry>,
	source:&PathBuf,
	target:&PathBuf,
	overwrite:bool,
) -> Result<(), CommonError> {
	EnvUtils::IsPathAllowedForFilesystemAccess(app_handle, source).await?;
	EnvUtils::IsPathAllowedForFilesystemAccess(app_handle, target).await?;
	if !overwrite && fs::try_exists(target).await.unwrap_or(false) {
		return Err(CommonError::FileSystemFileExists(target.clone()));
	}
	fs::rename(source, target)
		.await
		.map_err(|e| EnvUtils::MapIoErrorToCommonError(e, source.clone(), "Rename"))
}

// Logic to copy a file. Does not support recursive directory copy.
pub async fn CopyLogic(
	app_handle:&AppHandle<Wry>,
	source:&PathBuf,
	target:&PathBuf,
	overwrite:bool,
) -> Result<(), CommonError> {
	EnvUtils::IsPathAllowedForFilesystemAccess(app_handle, source).await?;
	EnvUtils::IsPathAllowedForFilesystemAccess(app_handle, target).await?;
	let source_metadata = StatFileLogic(app_handle, source).await?;
	if (source_metadata.FileType & FileTypeDTO::Directory as u8) != 0 {
		return Err(CommonError::NotImplemented { FeatureName:"Recursive directory copy".to_string() });
	}
	if !overwrite && fs::try_exists(target).await.unwrap_or(false) {
		return Err(CommonError::FileSystemFileExists(target.clone()));
	}
	fs::copy(source, target)
		.await
		.map(|_| ())
		.map_err(|e| EnvUtils::MapIoErrorToCommonError(e, source.clone(), "Copy"))
}

// Logic to create an empty file.
pub async fn CreateFileLogic(app_handle:&AppHandle<Wry>, path:&PathBuf) -> Result<(), CommonError> {
	WriteFileLogic(app_handle, path, vec![], true, false).await
}
