// @module VineError
// @description Defines the specific, structured error types for all operations
// within the Vine gRPC Inter-Process Communication (IPC) system.
use std::sync::{MutexGuard, PoisonError};

use thiserror::Error;

// A comprehensive error enum for the Vine IPC layer.
//
// This enum uses `thiserror` to provide detailed, descriptive error messages
// for various failure scenarios, from I/O issues to gRPC transport errors.
#[derive(Debug, Error)]
pub enum VineError {
	// An error occurred during I/O operations with the sidecar process's
	// streams.
	#[error("Sidecar process I/O error: {0}")]
	IoError(#[from] std::io::Error),

	// An error related to the application-level communication protocol.
	#[error("Sidecar communication protocol error: {0}")]
	CommunicationProtocolError(String),

	// A gRPC client channel for the specified sidecar could not be found or is
	// not ready.
	#[error(
		"Sidecar process '{SidecarIdentifier}' not found or its gRPC client channel is not established/ready: \
		 {Details}"
	)]
	ClientChannelError { SidecarIdentifier:String, Details:String },

	// An RPC call to a sidecar failed with a specific gRPC status.
	#[error(
		"gRPC call to sidecar '{SidecarIdentifier}' (method: '{MethodName}') failed: {StatusCode} - {StatusMessage}"
	)]
	gRPCRequestFailed {
		SidecarIdentifier:String,
		MethodName:String,
		StatusCode:String,
		StatusMessage:String,
	},

	// An error occurred while serializing or deserializing a JSON payload for
	// a gRPC message.
	#[error("JSON serialization error for gRPC payload: {0}")]
	SerializationError(#[from] serde_json::Error),

	// A low-level error occurred in the `tonic` gRPC transport layer.
	#[error("Tonic transport error: {0}")]
	TonicTransportError(#[from] tonic::transport::Error),

	// A request to a sidecar did not receive a response within the specified
	// timeout.
	#[error(
		"Request to sidecar '{SidecarIdentifier}' (method: '{MethodName}') timed out after {TimeoutMilliseconds}ms"
	)]
	RequestTimeout { SidecarIdentifier:String, MethodName:String, TimeoutMilliseconds:u64 },

	// A shared state mutex was "poisoned," indicating a panic in another
	// thread while the lock was held.
	#[error("Internal state lock poisoned: {0}")]
	InternalLockError(String),
}

// Provides a convenient conversion from a `PoisonError` (from a failed Mutex
// lock) into a `VineError`.
impl<T> From<PoisonError<MutexGuard<'_, T>>> for VineError {
	fn from(Error:PoisonError<MutexGuard<'_, T>>) -> Self {
		VineError::InternalLockError(format!("Shared state lock poisoned: {}", Error))
	}
}
