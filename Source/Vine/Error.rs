//! # VineError
//!
//! Defines the specific, structured error types for all operations within the
//! Vine gRPC Inter-Process Communication (IPC) system.

use std::sync::{MutexGuard, PoisonError};

use http::uri::InvalidUri;
use thiserror::Error;

/// A comprehensive error enum for the Vine IPC layer.
#[derive(Debug, Error)]
pub enum VineError {
	/// A gRPC client channel for the specified sidecar could not be found or
	/// is not ready.
	#[error("Sidecar '{0}' not found or its gRPC client channel is not ready.")]
	ClientNotConnected(String),

	/// An RPC call to a sidecar failed with a specific gRPC status.
	#[error("gRPC call failed: {0}")]
	RPCError(String),

	/// An error occurred while serializing or deserializing a JSON payload.
	#[error("JSON serialization error for gRPC payload: {0}")]
	SerializationError(#[from] serde_json::Error),

	/// A low-level error occurred in the `tonic` gRPC transport layer.
	#[error("Tonic transport error: {0}")]
	TonicTransportError(#[from] tonic::transport::Error),

	/// A request did not receive a response within the specified timeout.
	#[error(
		"Request to sidecar '{SidecarIdentifier}' (method: '{MethodName}') timed out after {TimeoutMilliseconds}ms"
	)]
	RequestTimeout { SidecarIdentifier:String, MethodName:String, TimeoutMilliseconds:u64 },

	/// A shared state mutex was "poisoned," indicating a panic.
	#[error("Internal state lock poisoned: {0}")]
	InternalLockError(String),

	/// An error occurred from an invalid URI.
	#[error("Invalid URI: {0}")]
	InvalidUri(#[from] InvalidUri),
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for VineError {
	fn from(Error:PoisonError<MutexGuard<'_, T>>) -> Self {
		VineError::InternalLockError(format!("Shared state lock poisoned: {}", Error))
	}
}

impl From<tonic::Status> for VineError {
	fn from(status:tonic::Status) -> Self { VineError::RPCError(status.to_string()) }
}
