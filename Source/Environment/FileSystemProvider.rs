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
