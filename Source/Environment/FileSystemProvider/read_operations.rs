//! # FileSystemProvider - Read Operations
//!
//! Implementation of
//! [`FileSystemReader`](CommonLibrary::FileSystem::FileSystemReader) for
//! [`MountainEnvironment`](crate::MountainEnvironment::MountainEnvironment)
//!
//! Provides secure, validated filesystem read access with workspace trust
//! enforcement.

use std::path::PathBuf;

use CommonLibrary::{
	Error::CommonError::CommonError,
	FileSystem::DTO::{FileSystemStatDTO::FileSystemStatDTO, FileTypeDTO::FileTypeDTO},
};
use tokio::fs;

use super::super::{MountainEnvironment::MountainEnvironment, Utility};

/// Read operations implementation for MountainEnvironment
pub(super) async fn read_file_impl(env:&MountainEnvironment, path:&PathBuf) -> Result<Vec<u8>, CommonError> {
	Utility::IsPathAllowedForAccess(&env.ApplicationState, path)?;

	// Validate that the path exists and is a file, not a directory
	let metadata = fs::metadata(path)
		.await
		.map_err(|error| CommonError::FromStandardIOError(error, path.clone(), "ReadFile.Stat"))?;

	if metadata.is_dir() {
		return Err(CommonError::InvalidArgument {
			ArgumentName:"Path".to_string(),
			Reason:format!("Cannot read directory as file: {}", path.display()),
		});
	}

	fs::read(path)
		.await
		.map_err(|error| CommonError::FromStandardIOError(error, path.clone(), "ReadFile"))
}

/// Stat operations implementation for MountainEnvironment
pub(super) async fn stat_file_impl(env:&MountainEnvironment, path:&PathBuf) -> Result<FileSystemStatDTO, CommonError> {
	Utility::IsPathAllowedForAccess(&env.ApplicationState, path)?;

	let metadata = fs::metadata(path)
		.await
		.map_err(|error| CommonError::FromStandardIOError(error, path.clone(), "StatFile"))?;

	let mut file_type = 0_u8;

	if metadata.is_file() {
		file_type |= FileTypeDTO::File as u8;
	}

	if metadata.is_dir() {
		file_type |= FileTypeDTO::Directory as u8;
	}

	// Check for symbolic link separately using symlink_metadata()
	let file_type_raw = fs::symlink_metadata(path)
		.await
		.map_err(|error| CommonError::FromStandardIOError(error, path.clone(), "StatFile.FileType"))?;

	if file_type_raw.is_symlink() {
		file_type |= FileTypeDTO::SymbolicLink as u8;
	}

	// Note: Windows typically doesn't support creation_time, handle gracefully
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

		// Capture file permissions by extracting Unix file mode (st_mode) and Windows
		// file attributes. On Unix, extract permission bits (rwx for owner/group/others)
		// and store in FileSystemPermissionsDTO. On Windows, capture attributes
		// (readonly, hidden, system, archive). This enables preserving permissions
		// during file operations and respecting the user's filesystem ACLs. Currently
		// returns None, which defaults to inherited permissions.
		Permissions:None,
	})
}

/// ReadDirectory operations implementation for MountainEnvironment
pub(super) async fn read_directory_impl(
	env:&MountainEnvironment,
	path:&PathBuf,
) -> Result<Vec<(String, FileTypeDTO)>, CommonError> {
	Utility::IsPathAllowedForAccess(&env.ApplicationState, path)?;

	// Validate that the path exists and is a directory
	let metadata = fs::metadata(path)
		.await
		.map_err(|error| CommonError::FromStandardIOError(error, path.clone(), "ReadDirectory.Stat"))?;

	if !metadata.is_dir() {
		return Err(CommonError::InvalidArgument {
			ArgumentName:"Path".to_string(),
			Reason:format!("Cannot read directory: path is not a directory: {}", path.display()),
		});
	}

	let mut entries = Vec::new();

	let mut read_dir = fs::read_dir(path)
		.await
		.map_err(|error| CommonError::FromStandardIOError(error, path.clone(), "ReadDirectory"))?;

	while let Some(entry_result) = read_dir
		.next_entry()
		.await
		.map_err(|error| CommonError::FromStandardIOError(error, path.clone(), "ReadDirectory.NextEntry"))?
	{
		let file_name = entry_result.file_name().to_string_lossy().into_owned();

		// Determine file type including symbolic link detection
		let file_type = match entry_result.file_type().await {
			Ok(ft) => {
				if ft.is_symlink() {
					FileTypeDTO::SymbolicLink
				} else if ft.is_dir() {
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
