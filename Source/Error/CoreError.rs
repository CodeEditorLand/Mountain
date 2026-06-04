//! # Core Error Types (local, superseded)
//!
//! Base building blocks of Mountain's error taxonomy. Defines five
//! core types: [`ErrorSeverity`], [`ErrorKind`], [`ErrorContext`],
//! [`MountainError`], and a generic `Result<T>` alias. The five
//! per-domain sibling modules (`IPCError`, `FileSystemError`,
//! `ConfigurationError`, `ProviderError`, `ServiceError`) wrap an
//! `ErrorContext` and converge on `MountainError` via a `From` impl.
//!
//! ## Status
//!
//! Zero callers as of 2026-05-02. These types are superseded by
//! `CommonLibrary::Error::CommonError`. The module remains in place
//! so that a future migration back to per-domain error types can
//! pick up the existing constructors without rebuilding them. Do not
//! add new callers - use `CommonError` directly.

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

/// Severity level of an error, used to categorize the impact of
/// an error from informational to critical.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
	/// Informational: no action required.
	Info = 0,

	/// Warning: something unexpected but non-blocking.
	Warning = 1,

	/// Error: operation failed and requires attention.
	Error = 2,

	/// Critical: system-level failure, immediate action needed.
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

/// Top-level error category assigned to every error at construction
/// time. Used for routing, filtering, and aggregation in log sinks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
	/// Inter-process or inter-service communication failure.
	IPC,

	/// File or directory operation failure.
	FileSystem,

	/// Configuration read/write/validation failure.
	Configuration,

	/// Sidecar or long-running service lifecycle failure.
	Service,

	/// Capability provider (file, terminal, document, etc.) failure.
	Provider,

	/// Unclassified error.
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

/// Companion metadata attached to every [`MountainError`]. Carries
/// the human-readable message, categorization via [`ErrorKind`] and
/// [`ErrorSeverity`], and optional operation/component context for
/// log aggregation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
	/// Human-readable description of what went wrong.
	pub message:String,

	/// Category of the failure (e.g. `IPC`, `FileSystem`).
	pub kind:ErrorKind,

	/// Impact level (Info through Critical).
	pub severity:ErrorSeverity,

	/// Operation that was in progress when the error occurred, if known.
	pub operation:Option<String>,

	/// Component or module that raised the error, if known.
	pub component:Option<String>,
}

impl ErrorContext {
	/// Creates a new context with default kind and severity.
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

/// Base error type for Mountain, wrapping an [`ErrorContext`] with
/// optional raw source text and an optional stack trace snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountainError {
	/// Categorization, severity, and human-readable message.
	pub context:ErrorContext,

	/// Raw source text or underlying cause, if available.
	pub source:Option<String>,

	/// Stack trace captured at the error site, if available.
	pub stack_trace:Option<String>,
}

impl MountainError {
	/// Creates a new error from the given context.
	pub fn new(context:ErrorContext) -> Self { Self { context, source:None, stack_trace:None } }

	/// Attaches a raw source string (e.g. the underlying error's display
	/// output).
	pub fn with_source(mut self, source:impl Into<String>) -> Self {
		self.source = Some(source.into());

		self
	}

	/// Attaches a stack trace snapshot for post-mortem debugging.
	pub fn with_stack_trace(mut self, stack_trace:impl Into<String>) -> Self {
		self.stack_trace = Some(stack_trace.into());

		self
	}

	/// Returns the human-readable message.
	pub fn message(&self) -> &str { &self.context.message }

	/// Returns the error kind.
	pub fn kind(&self) -> ErrorKind { self.context.kind }

	/// Returns the severity level.
	pub fn severity(&self) -> ErrorSeverity { self.context.severity }

	/// Returns `true` when the severity is `Critical`.
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
