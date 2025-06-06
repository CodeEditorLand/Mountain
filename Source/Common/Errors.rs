// File: Common/Errors.rs
// Defines the universal error enum for the project, used across Mountain and
// potentially serialized for communication. This consolidates various error
// kinds into a single, manageable type.

#![allow(non_snake_case, non_camel_case_types)]

use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum CommonError {
	// Filesystem Errors
	#[error("Filesystem I/O error for '{Path}': {Description}")]
	FsIo { Path:PathBuf, Description:String },
	#[error("Resource not found: {0}")]
	FsNotFound(PathBuf),
	#[error("Permission denied for operation on '{Path}': {Reason}")]
	FsPermissionDenied { Path:PathBuf, Reason:String },
	#[error("Resource already exists: {0}")]
	FsFileExists(PathBuf),
	#[error("Path is not a directory: {0}")]
	FsNotADirectory(PathBuf),
	#[error("Path is a directory (expected a file): {0}")]
	FsIsADirectory(PathBuf),
	#[error("Directory not empty: {0}")]
	FsNotEmpty(PathBuf),
	#[error("Read error for '{Path}': {Description}")]
	FsRead { Path:PathBuf, Description:String },
	#[error("Write error for '{Path}': {Description}")]
	FsWrite { Path:PathBuf, Description:String },
	#[error("Stat error for '{Path}': {Description}")]
	FsStat { Path:PathBuf, Description:String },
	#[error("ReadDir error for '{Path}': {Description}")]
	FsReadDir { Path:PathBuf, Description:String },
	#[error("Mkdir error for '{Path}': {Description}")]
	FsMkdir { Path:PathBuf, Description:String },
	#[error("Delete error for '{Path}': {Description}")]
	FsDelete { Path:PathBuf, Description:String },
	#[error("Rename error from '{Source}' to '{Target}': {Description}")]
	FsRename { Source:PathBuf, Target:PathBuf, Description:String },
	#[error("Copy error from '{Source}' to '{Target}': {Description}")]
	FsCopy { Source:PathBuf, Target:PathBuf, Description:String },

	// Configuration Errors
	#[error("Configuration update error for key '{Key}': {Description}")]
	ConfigUpdate { Key:String, Description:String },
	#[error("Configuration load error: {Description}")]
	ConfigLoad { Description:String },

	// State & Argument Errors
	#[error("Invalid argument '{ArgumentName}': {Reason}")]
	InvalidArg { ArgumentName:String, Reason:String },
	#[error("Internal state access error (e.g., lock poisoned): {Context}")]
	StateLock { Context:String },

	// IPC & Command Errors
	#[error("Inter-process communication error: {Description}")]
	IpcError { Description:String },
	#[error("Command '{CommandIdentifier}' execution failed: {Reason}")]
	CommandExecution { CommandIdentifier:String, Reason:String },
	#[error("Command '{CommandIdentifier}' registration failed: {Reason}")]
	CommandRegistration { CommandIdentifier:String, Reason:String },
	#[error("Failed to list commands: {Reason}")]
	CommandList { Reason:String },

	// Provider Errors
	#[error("Language provider registration failed for type '{ProviderType}': {Reason}")]
	ProviderRegistration { ProviderType:String, Reason:String },
	#[error("Language provider '{ProviderIdentifier}' invocation failed: {Reason}")]
	ProviderInvocation { ProviderIdentifier:String, Reason:String },
	#[error("No provider found for feature '{Feature}' on document '{DocumentUri}'")]
	ProviderNotFound { Feature:String, DocumentUri:String },

	// Service-specific Errors
	#[error("Secret access for key '{Key}' failed: {Reason}")]
	SecretsAccess { Key:String, Reason:String },
	#[error("Output channel '{ChannelName}' operation failed: {Reason}")]
	OutputChannel { ChannelName:String, Reason:String },
	#[error("Diagnostics operation failed: {Reason}")]
	Diagnostics { Reason:String },
	#[error("UI interaction failed: {Reason}")]
	UiInteraction { Reason:String },

	// General & Fallback Errors
	#[error("Feature not implemented: {FeatureName}")]
	NotImplemented { FeatureName:String },
	#[error("Serialization or Deserialization error: {Description}")]
	SerdeError { Description:String },
	#[error("An unknown internal error occurred: {Description}")]
	Unknown { Description:String },
}

impl CommonError {
	/// Creates a `CommonError` from a standard `std::io::Error`.
	pub fn FromStdIoError(StdIoError:std::io::Error, Path:PathBuf, OperationContext:&str) -> Self {
		let Description = StdIoError.to_string();
		match StdIoError.kind() {
			std::io::ErrorKind::NotFound => CommonError::FsNotFound(Path),
			std::io::ErrorKind::PermissionDenied => CommonError::FsPermissionDenied { Path, Reason:Description },
			std::io::ErrorKind::AlreadyExists => CommonError::FsFileExists(Path),
			_ => CommonError::FsIo { Path, Description:format!("{} failed: {}", OperationContext, Description) },
		}
	}
}

impl From<serde_json::Error> for CommonError {
	/// Converts a `serde_json::Error` into a `CommonError::SerdeError`.
	fn from(SerdeError:serde_json::Error) -> Self { CommonError::SerdeError { Description:SerdeError.to_string() } }
}
