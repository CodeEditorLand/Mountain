//! # File System Error Types
//!
//! Provides file system operation error types for Mountain.
//! Used for all file system related errors.

use std::{error::Error as StdError, fmt, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

/// File system operation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileSystemError {
	/// File not found
	FileNotFound { context:ErrorContext, path:PathBuf },

	/// Permission denied
	PermissionDenied { context:ErrorContext, path:PathBuf },

	/// I/O error occurred
	IOError { context:ErrorContext, path:Option<PathBuf>, operation:String },

	/// Invalid path
	InvalidPath { context:ErrorContext, path:PathBuf },

	/// Directory not empty
	DirectoryNotEmpty { context:ErrorContext, path:PathBuf },

	/// File already exists
	FileAlreadyExists { context:ErrorContext, path:PathBuf },

	/// Not a directory
	NotADirectory { context:ErrorContext, path:PathBuf },

	/// Not a file
	NotAFile { context:ErrorContext, path:PathBuf },
}

impl FileSystemError {
	/// Get the error context
	pub fn context(&self) -> &ErrorContext {
		match self {
			FileSystemError::FileNotFound { context, .. } => context,

			FileSystemError::PermissionDenied { context, .. } => context,

			FileSystemError::IOError { context, .. } => context,

			FileSystemError::InvalidPath { context, .. } => context,

			FileSystemError::DirectoryNotEmpty { context, .. } => context,

			FileSystemError::FileAlreadyExists { context, .. } => context,

			FileSystemError::NotADirectory { context, .. } => context,

			FileSystemError::NotAFile { context, .. } => context,
		}
	}

	/// Create a file not found error
	pub fn file_not_found(path:impl Into<PathBuf>) -> Self {
		let path = path.into();

		Self::FileNotFound {
			context:ErrorContext::new(format!("File not found: {}", path.display()))
				.with_kind(ErrorKind::FileSystem)
				.with_severity(ErrorSeverity::Error),

			path,
		}
	}

	/// Create a permission denied error
	pub fn permission_denied(path:impl Into<PathBuf>) -> Self {
		let path = path.into();

		Self::PermissionDenied {
			context:ErrorContext::new(format!("Permission denied: {}", path.display()))
				.with_kind(ErrorKind::FileSystem)
				.with_severity(ErrorSeverity::Error),

			path,
		}
	}

	/// Create an I/O error
	pub fn io_error(operation:impl Into<String>, path:Option<PathBuf>, message:impl Into<String>) -> Self {
		let operation_str = operation.into();

		Self::IOError {
			context:ErrorContext::new(message)
				.with_kind(ErrorKind::FileSystem)
				.with_severity(ErrorSeverity::Error)
				.with_operation(operation_str.clone()),

			path,

			operation:operation_str,
		}
	}

	/// Create an invalid path error
	pub fn invalid_path(path:impl Into<PathBuf>) -> Self {
		let path = path.into();

		Self::InvalidPath {
			context:ErrorContext::new(format!("Invalid path: {}", path.display()))
				.with_kind(ErrorKind::FileSystem)
				.with_severity(ErrorSeverity::Error),

			path,
		}
	}

	/// Get the affected path
	pub fn path(&self) -> Option<&PathBuf> {
		match self {
			FileSystemError::FileNotFound { path, .. } => Some(path),

			FileSystemError::PermissionDenied { path, .. } => Some(path),

			FileSystemError::IOError { path, .. } => path.as_ref(),

			FileSystemError::InvalidPath { path, .. } => Some(path),

			FileSystemError::DirectoryNotEmpty { path, .. } => Some(path),

			FileSystemError::FileAlreadyExists { path, .. } => Some(path),

			FileSystemError::NotADirectory { path, .. } => Some(path),

			FileSystemError::NotAFile { path, .. } => Some(path),
		}
	}
}

impl fmt::Display for FileSystemError {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.context())?;

		if let Some(path) = self.path() {
			write!(f, " (path: {})", path.display())?;
		}

		Ok(())
	}
}

impl StdError for FileSystemError {}

impl From<FileSystemError> for MountainError {
	fn from(err:FileSystemError) -> Self { MountainError::new(err.context().clone()).with_source(err.to_string()) }
}

impl From<std::io::Error> for FileSystemError {
	fn from(err:std::io::Error) -> Self { Self::io_error("I/O operation", None, err.to_string()) }
}
