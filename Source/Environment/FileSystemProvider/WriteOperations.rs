//! # FileSystemProvider - Write Operations
//!
//! Implementation of
//! [`FileSystemWriter`](CommonLibrary::FileSystem::FileSystemWriter) for
//! [`MountainEnvironment`]
//!
//! Provides secure, validated filesystem write access with workspace trust
//! enforcement.

use std::path::PathBuf;

use CommonLibrary::{Error::CommonError::CommonError, FileSystem::DTO::FileTypeDTO::FileTypeDTO};
use tokio::fs;

use super::super::{MountainEnvironment::MountainEnvironment, Utility};

/// Write operations implementation for MountainEnvironment
pub(super) async fn write_file_impl(
	env:&MountainEnvironment,
	path:&PathBuf,
	content:Vec<u8>,
	create:bool,
	overwrite:bool,
) -> Result<(), CommonError> {
	Utility::IsPathAllowedForAccess(&env.ApplicationState, path)?;

	// Validate that Content is not excessively large to prevent memory issues
	if content.len() > 1024 * 1024 * 1024 {
		// 1 GB limit
		return Err(CommonError::InvalidArgument {
			ArgumentName:"Content".to_string(),
			Reason:"Content exceeds maximum size limit of 1GB".to_string(),
		});
	}

	let path_exists = fs::try_exists(path).await.unwrap_or(false);

	if path_exists && !overwrite {
		return Err(CommonError::FileSystemFileExists(path.clone()));
	}

	if !path_exists && !create {
		return Err(CommonError::FileSystemNotFound(path.clone()));
	}

	// Create parent directories if they don't exist
	if let Some(parent_directory) = path.parent() {
		if !fs::try_exists(parent_directory).await.unwrap_or(false) {
			fs::create_dir_all(parent_directory).await.map_err(|error| {
				CommonError::FromStandardIOError(error, parent_directory.to_path_buf(), "WriteFile.CreateParent")
			})?;
		}
	}

	fs::write(path, &content)
		.await
		.map_err(|error| CommonError::FromStandardIOError(error, path.clone(), "WriteFile"))?;

	// Implement atomic write pattern to prevent partial writes and data corruption
	// on crashes or interrupts. The current implementation writes directly to the
	// target file, which can leave corrupted files if the operation is interrupted.
	// A robust implementation: 1) writes content to a temporary file in the same
	// directory (ensuring same filesystem for atomic rename), 2) flushes and syncs
	// the temporary file to disk (fsync), 3) atomically renames the temporary file
	// to the target path using fs::rename (POSIX rename is atomic within a
	// filesystem), 4) deletes old file if replacing, or handles temp cleanup on
	// failure. This pattern ensures the target file is either fully written or
	// unchanged.
	Ok(())
}

/// CreateDirectory operations implementation for MountainEnvironment
pub(super) async fn create_directory_impl(
	env:&MountainEnvironment,
	path:&PathBuf,
	recursive:bool,
) -> Result<(), CommonError> {
	Utility::IsPathAllowedForAccess(&env.ApplicationState, path)?;

	// Validate that parent path doesn't point to a file
	if let Some(parent_path) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
		if fs::try_exists(parent_path).await.unwrap_or(false) {
			let parent_metadata = fs::metadata(parent_path).await.map_err(|error| {
				CommonError::FromStandardIOError(error, parent_path.to_path_buf(), "CreateDirectory.ParentStat")
			})?;

			if parent_metadata.is_file() {
				return Err(CommonError::InvalidArgument {
					ArgumentName:"Path".to_string(),
					Reason:format!("Cannot create directory: parent path is a file: {}", parent_path.display()),
				});
			}
		}
	}

	let operation = if recursive {
		fs::create_dir_all(path).await
	} else {
		fs::create_dir(path).await
	};

	operation.map_err(|error| CommonError::FromStandardIOError(error, path.clone(), "CreateDirectory"))
}

/// Delete operations implementation for MountainEnvironment
pub(super) async fn delete_impl(
	env:&MountainEnvironment,
	path:&PathBuf,
	recursive:bool,
	_use_trash:bool,
) -> Result<(), CommonError> {
	Utility::IsPathAllowedForAccess(&env.ApplicationState, path)?;

	// A full implementation would use the `trash` crate if `UseTrash` is true.
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

			operation.map_err(|error| CommonError::FromStandardIOError(error, path.clone(), "Delete"))
		},

		// Idempotent success
		Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),

		Err(error) => Err(CommonError::FromStandardIOError(error, path.clone(), "Delete.Stat")),
	}
}

/// Rename operations implementation for MountainEnvironment
pub(super) async fn rename_impl(
	env:&MountainEnvironment,
	source:&PathBuf,
	target:&PathBuf,
	overwrite:bool,
) -> Result<(), CommonError> {
	Utility::IsPathAllowedForAccess(&env.ApplicationState, source)?;

	Utility::IsPathAllowedForAccess(&env.ApplicationState, target)?;

	if !overwrite && fs::try_exists(target).await.unwrap_or(false) {
		return Err(CommonError::FileSystemFileExists(target.clone()));
	}

	fs::rename(source, target)
		.await
		.map_err(|error| CommonError::FromStandardIOError(error, source.clone(), "Rename"))
}

/// Copy operations implementation for MountainEnvironment
pub(super) async fn copy_impl(
	env:&MountainEnvironment,
	source:&PathBuf,
	target:&PathBuf,
	overwrite:bool,
) -> Result<(), CommonError> {
	Utility::IsPathAllowedForAccess(&env.ApplicationState, source)?;

	Utility::IsPathAllowedForAccess(&env.ApplicationState, target)?;

	// Validate that source exists
	if !fs::try_exists(source).await.unwrap_or(false) {
		return Err(CommonError::FileSystemNotFound(source.clone()));
	}

	// Call stat_file_impl from the read_operations module
	let source_metadata = super::read_operations::stat_file_impl(env, source).await?;

	if (source_metadata.FileType & FileTypeDTO::Directory as u8) != 0 {
		return Err(CommonError::NotImplemented { FeatureName:"Recursive directory copy".to_string() });
	}

	// Prevent copying file to itself (which would truncate it)
	if fs::canonicalize(source).await.ok().as_ref() == fs::canonicalize(target).await.ok().as_ref() {
		return Err(CommonError::InvalidArgument {
			ArgumentName:"Target".to_string(),
			Reason:"Cannot copy file to itself".to_string(),
		});
	}

	if !overwrite && fs::try_exists(target).await.unwrap_or(false) {
		return Err(CommonError::FileSystemFileExists(target.clone()));
	}

	// Create target parent directory if needed
	if let Some(target_parent) = target.parent() {
		if !fs::try_exists(target_parent).await.unwrap_or(false) {
			fs::create_dir_all(target_parent).await.map_err(|error| {
				CommonError::FromStandardIOError(error, target_parent.to_path_buf(), "Copy.CreateTargetParent")
			})?;
		}
	}

	fs::copy(source, target)
		.await
		.map(|_| ())
		.map_err(|error| CommonError::FromStandardIOError(error, source.clone(), "Copy"))
}

/// CreateFile operations implementation for MountainEnvironment
pub(super) async fn create_file_impl(env:&MountainEnvironment, path:&PathBuf) -> Result<(), CommonError> {
	// Use WriteFile with an empty Vec, ensuring creation without overwrite.
	// This ensures proper parent directory creation and path validation.
	write_file_impl(env, path, vec![], true, false).await
}
