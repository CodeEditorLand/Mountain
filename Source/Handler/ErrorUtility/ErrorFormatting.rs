// @module ErrorFormatting
// @description Defines utility functions for creating standardized,
// serializable error strings for use in RPC and Tauri command responses.

use Common::error::CommonError;
use log::error;
use serde_json::json;

/// Creates a JSON-formatted error string from a message and an optional code.
/// This is the base error format returned to the frontend or sidecars.
pub fn RPCErrorString(message:String, code:Option<&str>) -> String {
	let error_code = code.unwrap_or("EUNKNOWN");
	error!("[RPCError] Code: '{}', Message: '{}'", error_code, message);
	// Note: In a production app, the error might be structured differently,
	// but a simple string is often sufficient for `Result<_, String>`.
	json!({ "Message": message, "Code": error_code }).to_string()
}

/// Creates a JSON-formatted error string specifically for parameter validation
/// failures.
pub fn RPCParamErrorString(method_name:&str, parameter_name:&str, expected_type:&str, index:Option<usize>) -> String {
	let base_message = format!(
		"Missing or invalid parameter '{}' (expected {}) for method '{}'",
		parameter_name, expected_type, method_name
	);
	let full_message = if let Some(i) = index {
		format!("{} at argument index {}.", base_message, i)
	} else {
		base_message
	};
	RPCErrorString(full_message, Some("EBADARG"))
}

/// Maps a structured `CommonError` enum variant to a standardized, serializable
/// RPC error string. This provides consistent error reporting across the
/// application.
pub fn MapCommonErrorToRPCString(error:CommonError, operation_context:&str) -> String {
	error!(
		"[CommonError Mapping] Operation '{}' resulted in error: {:?}",
		operation_context, error
	);

	let (message, code_string) = match error {
		// Filesystem Errors
		CommonError::FsNotFound(path) => (format!("Resource not found: {}", path.display()), "ENOENT"),
		CommonError::FsPermissionDenied { path, reason } => {
			(format!("Permission denied for '{}': {}", path.display(), reason), "EACCES")
		},
		CommonError::FsFileExists(path) => (format!("Resource already exists: {}", path.display()), "EEXIST"),
		CommonError::FsIo { path, description } => {
			(format!("I/O error on '{}': {}", path.display(), description), "EIO")
		},

		// Argument and State Errors
		CommonError::InvalidArg { argument_name, reason } => {
			(format!("Invalid argument '{}': {}", argument_name, reason), "EBADARG")
		},
		CommonError::StateLock { context } => (format!("Internal state error: {}", context), "ESTATELOCK"),

		// IPC Errors
		CommonError::IpcError { description } => {
			(format!("Inter-process communication error: {}", description), "EIPC")
		},

		// Command Errors
		CommonError::CommandExecution { command_identifier, reason } => {
			(
				format!("Command '{}' execution failed: {}", command_identifier, reason),
				"ECMDEXEC",
			)
		},
		CommonError::CommandNotFound { feature: _, identifier } => {
			(format!("Command not found: '{}'", identifier), "ECMDNOTFOUND")
		},

		// User Interface Errors
		CommonError::UiInteraction { reason } => (format!("User Interface interaction failed: {}", reason), "EUI"),

		// Default/Catch-all
		_ => {
			(
				format!("An unmapped internal error occurred during '{}': {}", operation_context, error),
				"EUNMAPPED",
			)
		},
	};

	RPCErrorString(message, Some(code_string))
}
