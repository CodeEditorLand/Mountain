//! `CoreError` - base error types for Mountain.

pub mod New;
pub mod WithKind;
pub mod WithSeverity;
pub mod WithOperation;
pub mod WithComponent;
pub mod WithSource;
pub mod WithStackTrace;
pub mod Message;
pub mod Kind;
pub mod Severity;
pub mod IsCritical;

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

/// Severity level of an error.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
	Info = 0,
	Warning = 1,
	Error = 2,
	Critical = 3,
}

/// Top-level error category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
	IPC,
	FileSystem,
	Configuration,
	Service,
	Provider,
	Other,
}

/// Companion metadata attached to every [`MountainError`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
	pub message:String,
	pub kind:ErrorKind,
	pub severity:ErrorSeverity,
	pub operation:Option<String>,
	pub component:Option<String>,
}

/// Base error type for Mountain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountainError {
	pub context:ErrorContext,
	pub source:Option<String>,
	pub stack_trace:Option<String>,
}

/// Result type alias for Mountain operations.
pub type Result<T> = std::result::Result<T, MountainError>;

pub type Struct = MountainError;
