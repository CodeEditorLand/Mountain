//! # FileSystemProvider (Environment)
//!
//! RESPONSIBILITIES:
//! - Implements
//!   [`FileSystemReader`](CommonLibrary::FileSystem::FileSystemReader) and
//!   [`FileSystemWriter`](CommonLibrary::FileSystem::FileSystemWriter) for
//! `MountainEnvironment`
//! - Provides secure, validated filesystem access with workspace trust
//!   enforcement
//! - Handles file operations: read, write, stat, delete, rename, copy,
//!   directory traversal
//! - Detects and handles symbolic links properly
//! - Enforces path validation to prevent directory traversal attacks
//!
//! SECURITY MODEL:
//! - Sandboxed filesystem access limited to registered workspace folders
//! - All operations call `Utility::PathSecurity::IsPathAllowedForAccess` first
//! - Requires workspace trust to be enabled for any file access
//! - Path normalization prevents `../` attacks
//! - Symbolic link detection avoids following untrusted links outside
//!   workspaces
//!
//! ERROR HANDLING:
//! - Uses [`CommonError`](CommonLibrary::Error::CommonError) for all operations
//! - File operation errors are mapped via `CommonError::FromStandardIOError`
//! - Validates paths are within workspace boundaries (IsPathAllowedForAccess)
//! - Rejects directory reads when file expected (ReadFile)
//!
//! PERFORMANCE:
//! - Uses async tokio::fs for non-blocking I/O operations
//! - Symbolic link detection uses `symlink_metadata` in addition to `metadata`
//! - TODO: Consider caching file metadata for frequently accessed files
//!
//! VS CODE REFERENCE:
//! - `vs/workbench/services/files/electron-browser/diskFileSystemProvider.ts` -
//!   secure FS access
//! - `vs/platform/files/common/files.ts` - file system interfaces
//! - `vs/base/common/network.ts` - URI and path handling
//!
//! TODO:
//! - Implement filesystem change watching (notify, inotify, FSEvents)
//! - Add path normalization to prevent directory traversal
//! - Implement proper symbolic link resolution with security checks
//! - Add support for file permissions and ownership metadata
//! - Implement atomic file writes using temp file + rename pattern
//! - Add filesystem usage statistics (disk space, file counts)
//! - Implement file attribute querying (hidden, readonly, executable)
//! - Add support for extended file attributes on Unix/macOS
//! - Consider adding filesystem cache for metadata
//! - Implement trash operation using platform trash API (not delete)
//! - Add support for file system encoding detection
//! - Implement case sensitivity handling based on filesystem type
//!
//! MODULE STRUCTURE:
//! - [`ReadOperations.rs`](ReadOperations.rs) - `FileSystemReader`
//!   implementation
//! - [`WriteOperations.rs`](WriteOperations.rs) - `FileSystemWriter`
//!   implementation

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
	/// Delegates to ReadOperations module
	async fn ReadFile(&self, path:&PathBuf) -> Result<Vec<u8>, CommonError> {
		ReadOperations::read_file_impl(self, path).await
	}

	/// Delegates to ReadOperations module
	async fn StatFile(&self, path:&PathBuf) -> Result<FileSystemStatDTO, CommonError> {
		ReadOperations::stat_file_impl(self, path).await
	}

	/// Delegates to ReadOperations module
	async fn ReadDirectory(&self, path:&PathBuf) -> Result<Vec<(String, FileTypeDTO)>, CommonError> {
		ReadOperations::read_directory_impl(self, path).await
	}
}

#[async_trait]
impl FileSystemWriter for MountainEnvironment {
	/// Delegates to WriteOperations module
	async fn WriteFile(&self, path:&PathBuf, content:Vec<u8>, create:bool, overwrite:bool) -> Result<(), CommonError> {
		WriteOperations::write_file_impl(self, path, content, create, overwrite).await
	}

	/// Delegates to WriteOperations module
	async fn CreateDirectory(&self, path:&PathBuf, recursive:bool) -> Result<(), CommonError> {
		WriteOperations::create_directory_impl(self, path, recursive).await
	}

	/// Delegates to WriteOperations module
	async fn Delete(&self, path:&PathBuf, recursive:bool, use_trash:bool) -> Result<(), CommonError> {
		WriteOperations::delete_impl(self, path, recursive, use_trash).await
	}

	/// Delegates to WriteOperations module
	async fn Rename(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		WriteOperations::rename_impl(self, source, target, overwrite).await
	}

	/// Delegates to WriteOperations module
	async fn Copy(&self, source:&PathBuf, target:&PathBuf, overwrite:bool) -> Result<(), CommonError> {
		WriteOperations::copy_impl(self, source, target, overwrite).await
	}

	/// Delegates to WriteOperations module
	async fn CreateFile(&self, path:&PathBuf) -> Result<(), CommonError> {
		WriteOperations::create_file_impl(self, path).await
	}
}
