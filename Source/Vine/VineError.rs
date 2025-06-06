
// Defines the specific error types related to the Vine IPC system,
// covering gRPC, process management, and communication protocol issues.

#![allow(non_snake_case, non_camel_case_types)]

use std::sync::{MutexGuard, PoisonError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum VineError {
	#[error("Sidecar process I/O error: {0}")]
	IoError(#[from] std::io::Error),

	#[error("Sidecar communication protocol error: {0}")]
	CommunicationProtocolError(String),

	#[error(
		"Sidecar process '{SidecarIdentifier}' not found or its gRPC client channel is not established/ready: \
		 {Details}"
	)]
	ClientChannelError { SidecarIdentifier:String, Details:String },

	#[error(
		"gRPC call to sidecar '{SidecarIdentifier}' (method: '{MethodName}') failed: {StatusCode} - {StatusMessage}"
	)]
	GrpcRequestFailed {
		SidecarIdentifier:String,
		MethodName:String,
		StatusCode:String,
		StatusMessage:String,
	},

	#[error("JSON serialization error for gRPC payload: {0}")]
	SerializationError(#[from] serde_json::Error),

	#[error("JSON deserialization error: {0}. Raw line (sample): '{1}'")]
	DeserializationError(String, String),

	#[error(
		"Request to sidecar '{SidecarIdentifier}' (method: '{MethodName}') timed out after \
		 {TimeoutDurationMilliseconds}ms"
	)]
	RequestTimeout { SidecarIdentifier:String, MethodName:String, TimeoutDurationMilliseconds:u64 },

	#[error("Internal state lock poisoned: {0}")]
	InternalLockError(String),

	#[error(
		"Request was cancelled because the sidecar's writer task failed or its communication channel was closed. \
		 Request ID: {RequestIdentifier}, Sidecar: '{SidecarIdentifier}'"
	)]
	RequestCancelledWriterTaskFailed { RequestIdentifier:u64, SidecarIdentifier:String },
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for VineError {
	fn from(Error:PoisonError<MutexGuard<'_, T>>) -> Self {
		VineError::InternalLockError(format!("Shared state lock poisoned: {}", Error))
	}
}
