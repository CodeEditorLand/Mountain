// File: Handlers/ErrorUtils/ErrorUtils.rs
// Defines utility functions for creating standardized RPC error strings.
// This ensures consistent error formatting for responses sent back to the
// sidecar.

#![allow(non_snake_case, non_camel_case_types)]

use Common::Errors::CommonError;
use log::error;
use serde_json::json;

/// Creates a JSON-formatted error string for RPC responses.
pub fn RpcErrorString(Message:String, Code:Option<&str>) -> String {
	let ErrorCode = Code.unwrap_or("EUNKNOWN_RPC_ERROR");

	if !(ErrorCode.eq_ignore_ascii_case("SUCCESS") || ErrorCode.is_empty()) {
		error!("[RpcError Creation] Code: '{}', Message: '{}'", ErrorCode, Message);
	}

	json!({
		"message": Message,
		"code": ErrorCode
	})
	.to_string()
}

/// Creates a JSON-formatted error string specifically for parameter validation
/// errors.
pub fn RpcParamErrorString(MethodName:&str, ParameterName:&str, ExpectedType:&str, Index:Option<usize>) -> String {
	let BaseMessage = format!(
		"Missing or invalid parameter '{}' (expected {}) for method/command '{}'",
		ParameterName, ExpectedType, MethodName
	);

	let FullMessage = if let Some(i) = Index {
		format!("{} at argument index {}.", BaseMessage, i)
	} else {
		BaseMessage
	};

	RpcErrorString(FullMessage, Some("EBADARG"))
}

/// Maps a `CommonError` enum variant to a standardized RPC error string.
pub fn MapCommonErrorToRpcString(Error:CommonError, OperationContext:&str) -> String {
	error!(
		"[CommonError Mapping] Operation '{}' resulted in CommonError: {:?}",
		OperationContext, Error
	);

	let (Message, CodeString) = match Error {
		CommonError::FsNotFound(Path) => (format!("Resource not found: {}", Path.display()), "ENOENT"),
		CommonError::FsPermissionDenied { Path, Reason } => {
			(format!("Permission denied for '{}': {}", Path.display(), Reason), "EACCES")
		},
		CommonError::FsFileExists(Path) => (format!("Resource already exists: {}", Path.display()), "EEXIST"),
		CommonError::FsNotADirectory(Path) => (format!("Path is not a directory: {}", Path.display()), "ENOTDIR"),
		CommonError::FsIsADirectory(Path) => (format!("Path is a directory: {}", Path.display()), "EISDIR"),
		CommonError::FsNotEmpty(Path) => (format!("Directory not empty: {}", Path.display()), "ENOTEMPTY"),
		CommonError::FsRead { Path, Description } => {
			(format!("Read error for '{}': {}", Path.display(), Description), "EIO_READ")
		},
		CommonError::FsWrite { Path, Description } => {
			(format!("Write error for '{}': {}", Path.display(), Description), "EIO_WRITE")
		},
		CommonError::FsStat { Path, Description } => {
			(format!("Stat error for '{}': {}", Path.display(), Description), "EIO_STAT")
		},
		CommonError::FsReadDir { Path, Description } => {
			(
				format!("ReadDir error for '{}': {}", Path.display(), Description),
				"EIO_READDIR",
			)
		},
		CommonError::FsMkdir { Path, Description } => {
			(format!("Mkdir error for '{}': {}", Path.display(), Description), "EIO_MKDIR")
		},
		CommonError::FsDelete { Path, Description } => {
			(format!("Delete error for '{}': {}", Path.display(), Description), "EIO_DELETE")
		},
		CommonError::FsRename { Source, Description, .. } => {
			(
				format!("Rename error for '{}': {}", Source.display(), Description),
				"EIO_RENAME",
			)
		},
		CommonError::FsCopy { Source, Description, .. } => {
			(format!("Copy error for '{}': {}", Source.display(), Description), "EIO_COPY")
		},
		CommonError::ConfigUpdate { Key, Description } => {
			(
				format!("Configuration update error for '{}': {}", Key, Description),
				"ECONFIGUPDATE",
			)
		},
		CommonError::ConfigLoad { Description } => (Description, "ECONFIGLOAD"),
		CommonError::InvalidArg { ArgumentName, Reason } => {
			(format!("Invalid argument '{}': {}", ArgumentName, Reason), "EBADARG")
		},
		CommonError::NotImplemented { FeatureName } => (format!("Feature not implemented: {}", FeatureName), "ENOSYS"),
		CommonError::StateLock { Context } => (format!("Internal state access error: {}", Context), "ESTATELOCK"),
		CommonError::IpcError { Description } => {
			(format!("Inter-process communication error: {}", Description), "EIPC")
		},
		CommonError::CommandExecution { CommandIdentifier, Reason } => {
			(
				format!("Command '{}' execution failed: {}", CommandIdentifier, Reason),
				"ECMDEXEC",
			)
		},
		CommonError::CommandRegistration { CommandIdentifier, Reason } => {
			(
				format!("Command '{}' registration failed: {}", CommandIdentifier, Reason),
				"ECMDREG",
			)
		},
		CommonError::CommandList { Reason } => (format!("Failed to list commands: {}", Reason), "ECMDLIST"),
		CommonError::SecretsAccess { Key, Reason } => {
			(format!("Secret access for key '{}' failed: {}", Key, Reason), "ESECRET")
		},
		CommonError::OutputChannel { ChannelName, Reason } => {
			(
				format!("Output channel '{}' operation failed: {}", ChannelName, Reason),
				"EOUTPUT",
			)
		},
		CommonError::Diagnostics { Reason } => (format!("Diagnostics operation failed: {}", Reason), "EDIAG"),
		CommonError::UiInteraction { Reason } => (format!("UI interaction failed: {}", Reason), "EUI"),
		CommonError::Unknown { Description } => (Description, "EUNKNOWN_INTERNAL"),
		_ => {
			(
				format!("An unmapped internal error occurred during '{}'", OperationContext),
				"EUNMAPPED",
			)
		},
	};

	RpcErrorString(Message, Some(CodeString))
}
