//! # IPC Error Types
//!
//! IPC-specific error types for Mountain.
//! Covers connection establishment, message send and receive,
//! format validation, operation timeout, permission checks,
//! service availability, and queue capacity.

use std::{error::Error as StdError, fmt};

use serde::{Deserialize, Serialize};

use super::CoreError::{ErrorContext, ErrorKind, ErrorSeverity, MountainError};

/// IPC-specific error types.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IPCError {

	/// Connection failed.
	ConnectionFailed { context:ErrorContext, source:Option<String> },

	/// Message send failed.
	MessageSendFailed { context:ErrorContext, message_id:Option<String> },

	/// Message receive failed.
	MessageReceiveFailed { context:ErrorContext, source:Option<String> },

	/// Invalid message format.
	InvalidMessageFormat { context:ErrorContext, raw_message:Option<String> },

	/// Timeout occurred.
	Timeout { context:ErrorContext, operation:Option<String>, timeout_ms:u64 },

	/// Permission denied.
	PermissionDenied { context:ErrorContext, required_permission:Option<String> },

	/// Service unavailable.
	ServiceUnavailable { context:ErrorContext, service_name:Option<String> },

	/// Queue overflow.
	QueueOverflow { context:ErrorContext, queue_size:usize },
}

impl IPCError {

	/// Get the error context.
	pub fn context(&self) -> &ErrorContext {
		match self {
			IPCError::ConnectionFailed { context, .. } => context,

			IPCError::MessageSendFailed { context, .. } => context,

			IPCError::MessageReceiveFailed { context, .. } => context,

			IPCError::InvalidMessageFormat { context, .. } => context,

			IPCError::Timeout { context, .. } => context,

			IPCError::PermissionDenied { context, .. } => context,

			IPCError::ServiceUnavailable { context, .. } => context,

			IPCError::QueueOverflow { context, .. } => context,
		}
	}

	/// Create a connection failed error.
	pub fn connection_failed(message:impl Into<String>) -> Self {
		Self::ConnectionFailed {
			context:ErrorContext::new(message)
				.with_kind(ErrorKind::IPC)
				.with_severity(ErrorSeverity::Error),

			source:None,
		}
	}

	/// Create a message send failed error.
	pub fn message_send_failed(message:impl Into<String>, message_id:Option<String>) -> Self {
		Self::MessageSendFailed {
			context:ErrorContext::new(message)
				.with_kind(ErrorKind::IPC)
				.with_severity(ErrorSeverity::Error),

			message_id,
		}
	}

	/// Create a timeout error.
	pub fn timeout(operation:impl Into<String>, timeout_ms:u64) -> Self {
		let operation_str = operation.into();

		Self::Timeout {
			context:ErrorContext::new(format!("Operation timed out after {}ms", timeout_ms))
				.with_kind(ErrorKind::IPC)
				.with_severity(ErrorSeverity::Error)
				.with_operation(operation_str.clone()),

			operation:Some(operation_str),

			timeout_ms,
		}
	}

	/// Create a permission denied error.
	pub fn permission_denied(message:impl Into<String>, required_permission:Option<String>) -> Self {
		Self::PermissionDenied {
			context:ErrorContext::new(message)
				.with_kind(ErrorKind::IPC)
				.with_severity(ErrorSeverity::Critical),

			required_permission,
		}
	}

	/// Create a service unavailable error.
	pub fn service_unavailable(message:impl Into<String>, service_name:Option<String>) -> Self {
		Self::ServiceUnavailable {
			context:ErrorContext::new(message)
				.with_kind(ErrorKind::IPC)
				.with_severity(ErrorSeverity::Error),

			service_name,
		}
	}
}

impl fmt::Display for IPCError {

	fn fmt(&self, f:&mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{}", self.context()) }
}

impl StdError for IPCError {}

impl From<IPCError> for MountainError {

	fn from(err:IPCError) -> Self { MountainError::new(err.context().clone()).with_source(err.to_string()) }
}
