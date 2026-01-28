//! This module follows the Land ecosystem's PascalCase naming convention.
//! See https://github.com/CodeEditorLand/Mountain/blob/main/Documentation/GitHub/Naming%20Conventions.md
//!
//! # VineError
//!
//! Defines the specific, structured error types for all operations within the
//! Vine gRPC Inter-Process Communication (IPC) system.

#![allow(non_snake_case, non_camel_case_types)]

use std::{
	net::AddrParseError,
	sync::{MutexGuard, PoisonError},
};

use http::uri::InvalidUri;
use thiserror::Error;

/// A comprehensive error enum for the Vine IPC layer.
#[derive(Debug, Error)]
pub enum VineError {
	/// A gRPC client channel for the specified sidecar could not be found or
	/// is not ready.
	#[error("SideCar '{0}' not found or its gRPC client channel is not ready.")]
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
		"Request to sidecar '{SideCarIdentifier}' (method: '{MethodName}') timed out after {TimeoutMilliseconds}ms"
	)]
	RequestTimeout { SideCarIdentifier:String, MethodName:String, TimeoutMilliseconds:u64 },

	/// A shared state mutex was "poisoned," indicating a panic.
	#[error("Internal state lock poisoned: {0}")]
	InternalLockError(String),

	/// An error occurred from an invalid URI.
	#[error("Invalid URI: {0}")]
	InvalidUri(#[from] InvalidUri),

	/// An error occurred while parsing a socket address.
	#[error("Invalid Socket Address: {0}")]
	AddressParseError(#[from] AddrParseError),
}

impl<T> From<PoisonError<MutexGuard<'_, T>>> for VineError {
	fn from(Error:PoisonError<MutexGuard<'_, T>>) -> Self {
		VineError::InternalLockError(format!("Shared state lock poisoned: {}", Error))
	}
}

impl From<tonic::Status> for VineError {
	fn from(status:tonic::Status) -> Self { VineError::RPCError(status.to_string()) }
}
