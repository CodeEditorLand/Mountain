//! # FileSystemProvider - Write Operations
//!
//! Implements [`FileSystemWriter`](CommonLibrary::FileSystem::FileSystemWriter)
//! for [`MountainEnvironment`]. Every function calls
//! `Utility::PathSecurity::IsPathAllowedForAccess` before any I/O.
//!
//! ## Functions
//!
//! - `write_file_impl` - writes bytes to a path. Enforces a 1 GB content guard,
//!   `create` / `overwrite` flag semantics (mirroring
//!   `vscode.workspace.fs.writeFile`), and auto-creates missing parent
//!   directories. Note: currently writes directly; the inline comment documents
//!   the planned atomic-rename pattern (`write-to-temp → fsync → rename`).
//! - `create_directory_impl` - creates a directory, optionally recursively.
//!   Validates that the parent is not a regular file.
//! - `delete_impl` - removes a file or directory. `recursive` controls
//!   `remove_dir_all` vs `remove_dir`. `_use_trash` is stubbed; the `trash`
//!   crate integration is planned. Idempotent: `NotFound` is treated as
//!   success.
//! - `rename_impl` - calls `tokio::fs::rename` (POSIX-atomic within a
//!   filesystem). Both source and target are path-security checked.
//! - `copy_impl` - copies a file or directory tree. Directories use the private
//!   `copy_directory_recursive` helper, which walks an explicit stack to avoid
//!   deep async-recursion stack overflows.
//! - `create_file_impl` - thin wrapper over `write_file_impl` with empty
//!   content, `create=true`, `overwrite=false`.

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

	Utility::PathSecurity::Fn(&env.ApplicationState, path)?;

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

	Utility::PathSecurity::Fn(&env.ApplicationState, path)?;

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

	Utility::PathSecurity::Fn(&env.ApplicationState, path)?;

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

	Utility::PathSecurity::Fn(&env.ApplicationState, source)?;

	Utility::PathSecurity::Fn(&env.ApplicationState, target)?;

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

	Utility::PathSecurity::Fn(&env.ApplicationState, source)?;

	Utility::PathSecurity::Fn(&env.ApplicationState, target)?;

	// Validate that source exists
	if !fs::try_exists(source).await.unwrap_or(false) {
		return Err(CommonError::FileSystemNotFound(source.clone()));
	}

	// Call stat_file_impl from the ReadOperations module
	let source_metadata = super::ReadOperations::stat_file_impl(env, source).await?;

	let SourceIsDir = (source_metadata.FileType & FileTypeDTO::Directory as u8) != 0;

	// Prevent copying file/dir to itself (which would truncate or
	// recursively explode).
	if fs::canonicalize(source).await.ok().as_ref() == fs::canonicalize(target).await.ok().as_ref() {
		return Err(CommonError::InvalidArgument {
			ArgumentName:"Target".to_string(),
			Reason:"Cannot copy file to itself".to_string(),
		});
	}

	if !overwrite && fs::try_exists(target).await.unwrap_or(false) {
		return Err(CommonError::FileSystemFileExists(target.clone()));
	}

	// Create target parent directory if needed (works for both file
	// and directory copies; the directory copy below also creates
	// the target itself).
	if let Some(target_parent) = target.parent() {
		if !fs::try_exists(target_parent).await.unwrap_or(false) {
			fs::create_dir_all(target_parent).await.map_err(|error| {
				CommonError::FromStandardIOError(error, target_parent.to_path_buf(), "Copy.CreateTargetParent")
			})?;
		}
	}

	if SourceIsDir {
		// Recursive directory copy. Walks the source tree iteratively
		// (avoids deep async recursion blowing the stack on
		// pathological depths) and re-creates each entry under the
		// target. Symlinks are followed to keep behaviour consistent
		// with VS Code's `IFileService.copy` - if you want preserve-
		// symlinks semantics, use `clone_native` instead which does a
		// COW reflink on supported filesystems.
		return copy_directory_recursive(source, target, overwrite).await;
	}

	fs::copy(source, target)
		.await
		.map(|_| ())
		.map_err(|error| CommonError::FromStandardIOError(error, source.clone(), "Copy"))
}

/// Recursively copy a directory tree from `source` into `target`.
/// Iterative (uses an explicit stack of `(SrcDir, DstDir)`) so it
/// can't blow the Tokio task stack on very deep trees. Files inside
/// re-use `tokio::fs::copy` for fast path; directories are created
/// with `create_dir`. Symlinks are dereferenced.
async fn copy_directory_recursive(source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {

	// Pre-create the top-level target dir.
	if !fs::try_exists(target).await.unwrap_or(false) {
		fs::create_dir(target)
			.await
			.map_err(|error| CommonError::FromStandardIOError(error, target.clone(), "Copy.CreateTargetRoot"))?;
	}

	let mut Stack:Vec<(PathBuf, PathBuf)> = vec![(source.clone(), target.clone())];

	while let Some((SrcDir, DstDir)) = Stack.pop() {
		let mut Entries = fs::read_dir(&SrcDir)
			.await
			.map_err(|error| CommonError::FromStandardIOError(error, SrcDir.clone(), "Copy.ReadDir"))?;

		while let Some(Entry) = Entries
			.next_entry()
			.await
			.map_err(|error| CommonError::FromStandardIOError(error, SrcDir.clone(), "Copy.NextEntry"))?
		{
			let Name = Entry.file_name();

			let SrcPath = SrcDir.join(&Name);

			let DstPath = DstDir.join(&Name);

			let FileType = Entry
				.file_type()
				.await
				.map_err(|error| CommonError::FromStandardIOError(error, SrcPath.clone(), "Copy.FileType"))?;

			if FileType.is_dir() {
				if !fs::try_exists(&DstPath).await.unwrap_or(false) {
					fs::create_dir(&DstPath).await.map_err(|error| {
						CommonError::FromStandardIOError(error, DstPath.clone(), "Copy.CreateSubDir")
					})?;
				}

				Stack.push((SrcPath, DstPath));
			} else {
				if !overwrite && fs::try_exists(&DstPath).await.unwrap_or(false) {
					return Err(CommonError::FileSystemFileExists(DstPath));
				}

				fs::copy(&SrcPath, &DstPath)
					.await
					.map_err(|error| CommonError::FromStandardIOError(error, SrcPath.clone(), "Copy.CopyFile"))?;
			}
		}
	}

	Ok(())
}

/// CreateFile operations implementation for MountainEnvironment
pub(super) async fn create_file_impl(env:&MountainEnvironment, path:&PathBuf) -> Result<(), CommonError> {

	// Use WriteFile with an empty Vec, ensuring creation without overwrite.
	// This ensures proper parent directory creation and path validation.
	write_file_impl(env, path, vec![], true, false).await
}
