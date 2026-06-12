//! # FileSystemProvider (Environment)
//!
//! Implements [`FileSystemReader`](CommonLibrary::FileSystem::FileSystemReader)
//! and [`FileSystemWriter`](CommonLibrary::FileSystem::FileSystemWriter) for
//! `MountainEnvironment`, providing secure, validated filesystem access with
//! workspace trust enforcement. Handles read, write, stat, delete, rename,
//! copy, and directory traversal; detects symbolic links.
//!
//! ## Security model
//!
//! All operations call `Utility::PathSecurity::IsPathAllowedForAccess` before
//! touching the filesystem. Access is sandboxed to registered workspace
//! folders, path normalization blocks `../` traversal, and symbolic links
//! outside the workspace are not followed.
//!
//! ## Implementation
//!
//! The trait impl is split across two sub-modules loaded via `#[path]`:
//! - `FileSystemProvider/ReadOperations.rs` - `FileSystemReader` impl
//! - `FileSystemProvider/WriteOperations.rs` - `FileSystemWriter` impl
//!
//! ## VS Code reference
//!
//! - `vs/workbench/services/files/electron-browser/diskFileSystemProvider.ts`
//! - `vs/platform/files/common/files.ts`
//! - `vs/base/common/network.ts`
//!
//! ## Planned Work
//!
//! - Filesystem change watching
//! - Path normalization enforcement
//! - Atomic writes via temp+rename
//! - File permissions/ownership metadata
//! - Extended attributes
//! - Trash API (not delete)
//! - Encoding detection
//! - Case-sensitivity handling
//! - Filesystem usage statistics
//! - Metadata caching

use std::path::PathBuf;

use CommonLibrary::{
	Error::CommonError::CommonError,
	FileSystem::{
		DTO::{FileSystemStatDTO::FileSystemStatDTO, FileTypeDTO::FileTypeDTO},
		FileSystemReader::FileSystemReader,
		FileSystemWriter::FileSystemWriter,
	},
};
use async_trait::async_trait;

use super::MountainEnvironment::MountainEnvironment;

// Private submodules containing the actual implementation
#[path = "FileSystemProvider/ReadOperations.rs"]
mod ReadOperations;

#[path = "FileSystemProvider/WriteOperations.rs"]
mod WriteOperations;

#[async_trait]
impl FileSystemReader for MountainEnvironment {
	/// Reads the full contents of a file at the given path.
	async fn ReadFile(&self, path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		ReadOperations::read_file_impl(self, path).await
	}

	/// Returns metadata (size, modification time, type) for a file at the
	/// given path.
	async fn StatFile(&self, path:&PathBuf) -> Result<FileSystemStatDTO, CommonError> {
		ReadOperations::stat_file_impl(self, path).await
	}

	/// Lists the entries (file names and types) in a directory at the given
	/// path.
	async fn ReadDirectory(&self, path:&PathBuf) -> Result<Vec<(String, FileTypeDTO)>, CommonError> {
		ReadOperations::read_directory_impl(self, path).await
	}
}

#[async_trait]
impl FileSystemWriter for MountainEnvironment {
	/// Writes content to a file, optionally creating or overwriting it.
	async fn WriteFile(&self, path:&PathBuf, content:Vec<u8>, create:bool, overwrite:bool) -> Result<(), CommonError> {
		WriteOperations::write_file_impl(self, path, content, create, overwrite).await
	}

	/// Creates a directory at the given path, optionally creating parent
	/// directories.
	async fn CreateDirectory(&self, path:&PathBuf, recursive:bool) -> Result<(), CommonError> {
		WriteOperations::create_directory_impl(self, path, recursive).await
	}

	/// Deletes a file or directory, optionally recursive and using the
	/// system trash.
	async fn Delete(&self, path:&PathBuf, recursive:bool, use_trash:bool) -> Result<(), CommonError> {
		WriteOperations::delete_impl(self, path, recursive, use_trash).await
	}

	/// Renames (moves) a file or directory from source to target, optionally
	/// overwriting the target.
	async fn Rename(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		WriteOperations::rename_impl(self, source, target, overwrite).await
	}

	/// Copies a file from source to target, optionally overwriting the
	/// target.
	async fn Copy(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		WriteOperations::copy_impl(self, source, target, overwrite).await
	}

	/// Creates an empty file at the given path.
	async fn CreateFile(&self, path:&PathBuf) -> Result<(), CommonError> {
		WriteOperations::create_file_impl(self, path).await
	}
}
