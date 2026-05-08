//! # Core Error Types (local, dead-code stack)
//!
//! Base building blocks of Mountain's local error taxonomy. Five
//! per-domain sibling modules (`IPCError`, `FileSystemError`,
//! `ConfigurationError`, `ProviderError`, `ServiceError`) wrap an
//! `ErrorContext` and converge on `MountainError` via a `From` impl.
//!
//! TODO atomic split: this file is **NOT** atomized into one-symbol-per-
//! file because the five sibling modules each construct
//! `ErrorContext { context: ..., severity: ..., kind: ... }` literally,
//! and call `.with_kind` / `.with_severity` / `.with_operation`. Splitting
//! `ErrorContext`/`MountainError` into `Struct`-renamed atoms would
//! require renaming every field+method across ~700 lines of dead code.
//! Defer until the whole stack is either deleted or migrated to
//! `CommonLibrary::Error::CommonError`.
//!
//! TODO: zero callers as of 2026-05-02 - superseded by
//! `CommonLibrary::Error::CommonError`.

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

/// Severity level of an error
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
	Info = 0,

	Warning = 1,

	Error = 2,

	Critical = 3,
}

impl fmt::Display for ErrorSeverity {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ErrorSeverity::Info => write!(f, "Info"),

			ErrorSeverity::Warning => write!(f, "Warning"),

			ErrorSeverity::Error => write!(f, "Error"),

			ErrorSeverity::Critical => write!(f, "Critical"),
		}
	}
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

impl fmt::Display for ErrorKind {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ErrorKind::IPC => write!(f, "IPC"),

			ErrorKind::FileSystem => write!(f, "FileSystem"),

			ErrorKind::Configuration => write!(f, "Configuration"),

			ErrorKind::Service => write!(f, "Service"),

			ErrorKind::Provider => write!(f, "Provider"),

			ErrorKind::Other => write!(f, "Other"),
		}
	}
}

/// Companion metadata attached to every `MountainError`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
	pub message:String,

	pub kind:ErrorKind,

	pub severity:ErrorSeverity,

	pub operation:Option<String>,

	pub component:Option<String>,
}

impl ErrorContext {
	pub fn new(message:impl Into<String>) -> Self {
		Self {
			message:message.into(),

			kind:ErrorKind::Other,

			severity:ErrorSeverity::Error,

			operation:None,

			component:None,
		}
	}

	pub fn with_kind(mut self, kind:ErrorKind) -> Self {
		self.kind = kind;

		self
	}

	pub fn with_severity(mut self, severity:ErrorSeverity) -> Self {
		self.severity = severity;

		self
	}

	pub fn with_operation(mut self, operation:impl Into<String>) -> Self {
		self.operation = Some(operation.into());

		self
	}

	pub fn with_component(mut self, component:impl Into<String>) -> Self {
		self.component = Some(component.into());

		self
	}
}

impl fmt::Display for ErrorContext {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "[{}][{}] {}", self.kind, self.severity, self.message)
	}
}

/// Base Mountain error type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountainError {
	pub context:ErrorContext,

	pub source:Option<String>,

	pub stack_trace:Option<String>,
}

impl MountainError {
	pub fn new(context:ErrorContext) -> Self { Self { context, source:None, stack_trace:None } }

	pub fn with_source(mut self, source:impl Into<String>) -> Self {
		self.source = Some(source.into());

		self
	}

	pub fn with_stack_trace(mut self, stack_trace:impl Into<String>) -> Self {
		self.stack_trace = Some(stack_trace.into());

		self
	}

	pub fn message(&self) -> &str { &self.context.message }

	pub fn kind(&self) -> ErrorKind { self.context.kind }

	pub fn severity(&self) -> ErrorSeverity { self.context.severity }

	pub fn is_critical(&self) -> bool { self.context.severity == ErrorSeverity::Critical }
}

impl fmt::Display for MountainError {
	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.context)?;

		if let Some(source) = &self.source {
			write!(f, " ({})", source)?;
		}

		Ok(())
	}
}

impl StdError for MountainError {}

impl From<ErrorContext> for MountainError {
	fn from(context:ErrorContext) -> Self { Self::new(context) }
}

/// Result type alias for Mountain operations.
pub type Result<T> = std::result::Result<T, MountainError>;
