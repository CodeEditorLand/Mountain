//! # Core Error Types
//!
//! Provides the base error types and traits used across Mountain.
//! All Mountain errors should implement or use these core types.

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

/// Severity level of an error
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ErrorSeverity {
	/// Informational, can be ignored
	Info = 0,
	/// Warning, might indicate a problem
	Warning = 1,
	/// Error, operation failed
	Error = 2,
	/// Critical, system may be unstable
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

/// Category of error
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorKind {
	/// IPC communication error
	IPC,
	/// File system error
	FileSystem,
	/// Configuration error
	Configuration,
	/// Service error
	Service,
	/// Provider error
	Provider,
	/// Generic/unknown error
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

/// Error context providing additional information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
	/// Human-readable error message
	pub message:String,
	/// Error kind
	pub kind:ErrorKind,
	/// Severity level
	pub severity:ErrorSeverity,
	/// Optional operation that caused the error
	pub operation:Option<String>,
	/// Optional component where the error occurred
	pub component:Option<String>,
}

impl ErrorContext {
	/// Create a new error context
	pub fn new(message:impl Into<String>) -> Self {
		Self {
			message:message.into(),
			kind:ErrorKind::Other,
			severity:ErrorSeverity::Error,
			operation:None,
			component:None,
		}
	}

	/// Set the error kind
	pub fn with_kind(mut self, kind:ErrorKind) -> Self {
		self.kind = kind;
		self
	}

	/// Set the severity level
	pub fn with_severity(mut self, severity:ErrorSeverity) -> Self {
		self.severity = severity;
		self
	}

	/// Set the operation
	pub fn with_operation(mut self, operation:impl Into<String>) -> Self {
		self.operation = Some(operation.into());
		self
	}

	/// Set the component
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

/// Base Mountain error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MountainError {
	/// Error context
	pub context:ErrorContext,
	/// Optional source error (simplified for serialization)
	pub source:Option<String>,
	/// Optional stack trace
	pub stack_trace:Option<String>,
}

impl MountainError {
	/// Create a new Mountain error
	pub fn new(context:ErrorContext) -> Self { Self { context, source:None, stack_trace:None } }

	/// Create an error with a source
	pub fn with_source(mut self, source:impl Into<String>) -> Self {
		self.source = Some(source.into());
		self
	}

	/// Create an error with a stack trace
	pub fn with_stack_trace(mut self, stack_trace:impl Into<String>) -> Self {
		self.stack_trace = Some(stack_trace.into());
		self
	}

	/// Get the error message
	pub fn message(&self) -> &str { &self.context.message }

	/// Get the error kind
	pub fn kind(&self) -> ErrorKind { self.context.kind }

	/// Get the error severity
	pub fn severity(&self) -> ErrorSeverity { self.context.severity }

	/// Check if error is critical
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

/// Result type alias for Mountain operations
pub type Result<T> = std::result::Result<T, MountainError>;
