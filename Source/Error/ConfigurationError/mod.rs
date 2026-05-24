//! `ConfigurationError`

pub mod Context;
pub mod KeyNotFound;
pub mod InvalidValue;
pub mod ValidationFailed;
pub mod ParseError;
pub mod FileNotFound;
pub mod CircularDependency;

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

/// Configuration operation error types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigurationError {
	/// Configuration key not found.
	KeyNotFound { context:ErrorContext, key:String, section:Option<String> },

	/// Invalid configuration value.
	InvalidValue { context:ErrorContext, key:String, expected_type:String },

	/// Configuration validation failed.
	ValidationFailed { context:ErrorContext, errors:Vec<String> },

	/// Configuration parse error.
	ParseError { context:ErrorContext, format:String, source:String },

	/// Configuration file not found.
	FileNotFound { context:ErrorContext, path:String },

	/// Configuration file read error.
	FileReadError { context:ErrorContext, path:String, source:String },

	/// Configuration file write error.
	FileWriteError { context:ErrorContext, path:String, source:String },

	/// Circular dependency detected.
	CircularDependency { context:ErrorContext, keys:Vec<String> },
}

pub type Struct = ConfigurationError;
