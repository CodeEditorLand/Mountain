// @module VineError
// @description Defines the specific, structured error types for all operations
// within the Vine gRPC Inter-Process Communication (IPC) system.

use std::sync::{MutexGuard, PoisonError};

use thiserror::Error;

/// A comprehensive error enum for the Vine IPC layer.
///
/// This enum uses `thiserror` to provide detailed, descriptive error messages
/// for various failure scenarios, from I/O issues to gRPC transport errors.
#[derive(Debug, Error)]
pub enum VineError {
	/// A gRPC client channel for the specified sidecar could not be found or is
	/// not ready.
	#[error(
		"Sidecar process '{sidecar_identifier}' not found or its gRPC client channel is not established/ready: \
		 {details}"
	)]
	ClientChannelError { sidecar_identifier:String, details:String },

	/// An RPC call to a sidecar failed with a specific gRPC status.
	#[error(
		"gRPC call to sidecar '{sidecar_identifier}' (method: '{method_name}') failed: {status_code} - \
		 {status_message}"
	)]
	gRPCRequestFailed {
		sidecar_identifier:String,
		method_name:String,
		status_code:String,
		status_message:String,
	},

	/// An error occurred while serializing or deserializing a JSON payload for
	/// a gRPC message.
	#[error("JSON serialization error for gRPC payload: {0}")]
	SerializationError(#[from] serde_json::Error),

	/// A low-level error occurred in the `tonic` gRPC transport layer.
	#[error("Tonic transport error: {0}")]
	TonicTransportError(#[from] tonic::transport::Error),

	/// A request to a sidecar did not receive a response within the specified
	/// timeout.
	#[error(
		"Request to sidecar '{sidecar_identifier}' (method: '{method_name}') timed out after {timeout_milliseconds}ms"
	)]
	RequestTimeout { sidecar_identifier:String, method_name:String, timeout_milliseconds:u64 },

	/// A shared state mutex was "poisoned," indicating a panic in another
	/// thread while the lock was held.
	#[error("Internal state lock poisoned: {0}")]
	InternalLockError(String),
}

/// Provides a convenient conversion from a `PoisonError` (from a failed Mutex
/// lock) into a `VineError`.
impl<T> From<PoisonError<MutexGuard<'_, T>>> for VineError {
	fn from(error:PoisonError<MutexGuard<'_, T>>) -> Self {
		VineError::InternalLockError(format!("Shared state lock poisoned: {}", error))
	}
}
