pub mod IsRecoverable;
pub mod ToTonicStatus;

use std::{
	net::AddrParseError,
	sync::{MutexGuard, PoisonError},
};

use http::uri::InvalidUri;
use thiserror::Error;

/// A comprehensive error enum for the Vine IPC layer.
///
/// Each variant provides detailed context about the failure, enabling
/// precise error handling and recovery strategies.
#[derive(Debug, Error)]
pub enum VineError {
	/// A gRPC client channel for the specified sidecar could not be found or
	/// is not ready in the connection pool.
	///
	/// This typically occurs when trying to send a request to a sidecar
	/// that was never connected or has been disconnected.
	#[error("SideCar '{0}' not found or its gRPC client channel is not ready.")]
	ClientNotConnected(String),

	/// Failed to establish a connection to the specified sidecar.
	///
	/// This indicates that connection attempts failed, possibly due to
	/// network issues, incorrect address, or the sidecar being unavailable.
	#[error("Failed to connect to sidecar '{SideCarIdentifier}' at '{Address}': {Reason}")]
	ConnectionFailed { SideCarIdentifier:String, Address:String, Reason:String },

	/// An established connection to the sidecar was lost.
	///
	/// This occurs when an active connection fails during communication.
	#[error("Connection to sidecar '{0}' was lost")]
	ConnectionLost(String),

	/// An RPC call to a sidecar failed with a specific gRPC status.
	///
	/// This wraps tonic::Status errors with more context about what went wrong.
	#[error("gRPC call failed: {0}")]
	RPCError(String),

	/// A request did not receive a response within the specified timeout.
	///
	/// Includes the sidecar identifier, method name, and timeout duration
	/// for debugging and retry logic.
	#[error(
		"Request to sidecar '{SideCarIdentifier}' (method: '{MethodName}') timed out after {TimeoutMilliseconds}ms"
	)]
	RequestTimeout { SideCarIdentifier:String, MethodName:String, TimeoutMilliseconds:u64 },

	/// A request was explicitly canceled before completion.
	#[error("Request to sidecar '{SideCarIdentifier}' (method: '{MethodName}') was canceled")]
	RequestCanceled { SideCarIdentifier:String, MethodName:String },

	/// An error occurred while serializing or deserializing a JSON payload.
	///
	/// This is automatically converted from serde_json::Error when using
	/// the ? operator.
	#[error("JSON serialization error for gRPC payload: {0}")]
	SerializationError(#[from] serde_json::Error),

	/// Message exceeded maximum allowed size.
	///
	/// This prevents denial-of-service attacks via oversized messages.
	#[error("Message size {ActualSize} bytes exceeds maximum allowed size {MaxSize} bytes")]
	MessageTooLarge { ActualSize:usize, MaxSize:usize },

	/// Message format validation failed.
	///
	/// This occurs when a message doesn't conform to expected structure.
	#[error("Invalid message format: {0}")]
	InvalidMessageFormat(String),

	/// A low-level error occurred in the `tonic` gRPC transport layer.
	///
	/// This is automatically converted from tonic::transport::Error.
	#[error("Tonic transport error: {0}")]
	TonicTransportError(#[from] tonic::transport::Error),

	/// A shared state mutex was \"poisoned,\" indicating a panic.
	///
	/// This is a critical error indicating that a thread panicked while
	/// holding a lock, leaving the shared state in an inconsistent state.
	#[error("Internal state lock poisoned: {0}")]
	InternalLockError(String),

	/// Invalid internal state detected.
	///
	/// This occurs when the system is in an unexpected state that should
	/// never happen during normal operation.
	#[error("Invalid internal state detected: {0}")]
	InvalidState(String),

	/// An error occurred from an invalid URI.
	///
	/// This is automatically converted from http::uri::InvalidUri.
	#[error("Invalid URI: {0}")]
	InvalidUri(#[from] InvalidUri),

	/// An error occurred while parsing a socket address.
	///
	/// This is automatically converted from std::net::AddrParseError.
	#[error("Invalid Socket Address: {0}")]
	AddressParseError(#[from] AddrParseError),
}

impl VineError {
	/// Checks if this error is recoverable (can retry the operation).
	///
	/// Recoverable errors include timeouts, connection issues, and temporary
	/// failures. Non-recoverable errors include serialization errors and
	/// invalid state.
	pub fn IsRecoverable(&self) -> bool {
		matches!(
			self,
			Self::RequestTimeout { .. }
				| Self::ConnectionFailed { .. }
				| Self::ConnectionLost(_)
				| Self::TonicTransportError(_)
		)
	}

	/// Converts the error to a tonic::Status for gRPC error responses.
	///
	/// This maps VineError variants to appropriate gRPC status codes:
	/// - RequestTimeout → DeadlineExceeded
	/// - ClientNotConnected → Unavailable
	/// - SerializationError → Internal
	/// - etc.
	pub fn ToTonicStatus(&self) -> tonic::Status {
		match self {
			Self::RequestTimeout { .. } => tonic::Status::deadline_exceeded(self.to_string()),

			Self::ClientNotConnected(_) | Self::ConnectionFailed { .. } => tonic::Status::unavailable(self.to_string()),

			Self::SerializationError(_) | Self::InternalLockError(_) | Self::InvalidState(_) => {
				tonic::Status::internal(self.to_string())
			},

			Self::MessageTooLarge { .. } => tonic::Status::resource_exhausted(self.to_string()),

			Self::InvalidMessageFormat(_) | Self::InvalidUri(_) | Self::AddressParseError(_) => {
				tonic::Status::invalid_argument(self.to_string())
			},

			Self::RequestCanceled { .. } => tonic::Status::cancelled(self.to_string()),

			Self::RPCError(msg) => tonic::Status::unknown(msg.clone()),

			Self::ConnectionLost(_) => tonic::Status::aborted(self.to_string()),

			Self::TonicTransportError(_) => tonic::Status::unavailable(self.to_string()),
		}
	}
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for VineError {
	fn from(Error:PoisonError<MutexGuard<'_, T>>) -> Self {
		VineError::InternalLockError(format!("Shared state lock poisoned: {}", Error))
	}
}

impl From<tonic::Status> for VineError {
	fn from(status:tonic::Status) -> Self {
		// Map gRPC status codes to appropriate VineError variants
		match status.code() {
			tonic::Code::DeadlineExceeded => VineError::RPCError(format!("Timeout: {}", status.message())),

			tonic::Code::NotFound => VineError::ClientNotConnected(status.message().to_string()),

			tonic::Code::AlreadyExists | tonic::Code::InvalidArgument | tonic::Code::OutOfRange => {
				VineError::InvalidMessageFormat(status.message().to_string())
			},

			tonic::Code::FailedPrecondition | tonic::Code::Aborted => {
				VineError::ConnectionLost(status.message().to_string())
			},

			tonic::Code::ResourceExhausted => VineError::MessageTooLarge { ActualSize:0, MaxSize:4 * 1024 * 1024 },

			tonic::Code::Cancelled => {
				VineError::RequestCanceled { SideCarIdentifier:"unknown".to_string(), MethodName:"unknown".to_string() }
			},

			tonic::Code::Unavailable => {
				VineError::ConnectionFailed {
					SideCarIdentifier:"unknown".to_string(),

					Address:"unknown".to_string(),

					Reason:status.message().to_string(),
				}
			},

			_ => VineError::RPCError(format!("{}: {}", status.code(), status.message())),
		}
	}
}
