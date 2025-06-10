/// @module ErrorFormatting
/// @description Defines utility functions for creating standardized,
/// serializable error strings for use in RPC and Tauri command responses.
use Common::error::CommonError;
use log::error;
use serde_json::json;

/// Creates a JSON-formatted error string from a message and an optional code.
/// This is the base error format returned to the frontend or sidecars.
pub fn RpcErrorString(Message:String, Code:Option<&str>) -> String {
	let ErrorCode = Code.unwrap_or("EUNKNOWN");
	error!("[RpcError] Code: '{}', Message: '{}'", ErrorCode, Message);
	// Note: In a production app, the error might be structured differently,
	// but a simple string is often sufficient for `Result<_, String>`.
	json!({ "Message": Message, "Code": ErrorCode }).to_string()
}

/// Creates a JSON-formatted error string specifically for parameter validation
/// failures.
pub fn RpcParamErrorString(MethodName:&str, ParameterName:&str, ExpectedType:&str, Index:Option<usize>) -> String {
	let BaseMessage = format!(
		"Missing or invalid parameter '{}' (expected {}) for method '{}'",
		ParameterName, ExpectedType, MethodName
	);
	let FullMessage = if let Some(i) = Index {
		format!("{} at argument index {}.", BaseMessage, i)
	} else {
		BaseMessage
	};
	RpcErrorString(FullMessage, Some("EBADARG"))
}

/// Maps a structured `CommonError` enum variant to a standardized, serializable
/// RPC error string. This provides consistent error reporting across the
/// application.
pub fn MapCommonErrorToRpcString(Error:CommonError, OperationContext:&str) -> String {
	error!(
		"[CommonError Mapping] Operation '{}' resulted in error: {:?}",
		OperationContext, Error
	);

	let (Message, CodeString) = match Error {
		// Filesystem Errors
		CommonError::FsNotFound(Path) => (format!("Resource not found: {}", Path.display()), "ENOENT"),
		CommonError::FsPermissionDenied { Path, Reason } => {
			(format!("Permission denied for '{}': {}", Path.display(), Reason), "EACCES")
		},
		CommonError::FsFileExists(Path) => (format!("Resource already exists: {}", Path.display()), "EEXIST"),
		CommonError::FsIo { Path, Description } => {
			(format!("I/O error on '{}': {}", Path.display(), Description), "EIO")
		},

		// Argument and State Errors
		CommonError::InvalidArg { ArgumentName, Reason } => {
			(format!("Invalid argument '{}': {}", ArgumentName, Reason), "EBADARG")
		},
		CommonError::StateLock { Context } => (format!("Internal state error: {}", Context), "ESTATELOCK"),

		// IPC Errors
		CommonError::IpcError { Description } => {
			(format!("Inter-process communication error: {}", Description), "EIPC")
		},

		// Command Errors
		CommonError::CommandExecution { CommandIdentifier, Reason } => {
			(
				format!("Command '{}' execution failed: {}", CommandIdentifier, Reason),
				"ECMDEXEC",
			)
		},
		CommonError::CommandNotFound { Feature, DocumentUri } => {
			(
				format!("Command not found for feature '{}' on resource '{}'", Feature, DocumentUri),
				"ECMDNOTFOUND",
			)
		},

		// UI Errors
		CommonError::UiInteraction { Reason } => (format!("UI interaction failed: {}", Reason), "EUI"),

		// Default/Catch-all
		_ => {
			(
				format!("An unmapped internal error occurred during '{}': {}", OperationContext, Error),
				"EUNMAPPED",
			)
		},
	};

	RpcErrorString(Message, Some(CodeString))
}
