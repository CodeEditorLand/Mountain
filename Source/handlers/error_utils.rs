// ---------------------------------------------------------------------------------------------
// Mountain Error Utilities (handlers/error_utils.rs)
// --------------------------------------------------------------------------------------------
// Provides shared utility functions for creating and formatting structured JSON
// error strings, typically used for RPC responses or Tauri command error
// results. Also includes helpers for mapping `CommonError` to these structured
// strings, ensuring consistent error reporting across the application.
//
// Responsibilities:
// - Defining a standard way to create JSON error strings (`rpc_error_string`).
// - Providing a specialized helper for parameter validation errors
//   (`rpc_param_error_string`).
// - Mapping internal `CommonError` types to user-facing JSON error strings
//   (`map_common_error_to_rpc_string`).
// - Centralizing error logging for these common error paths.
//
// Key Interactions:
// - Used by various handler modules (`handlers/*`) and RPC method
//   implementations (`rpc.rs`) to generate error responses for Track or Vine.
// - Consumes `CommonError` from the effect system or other operations.
// --------------------------------------------------------------------------------------------

use Land_Common::errors::CommonError;
// For logging errors when they are created/mapped
use log::error;
// Value might not be needed if only creating strings
use serde_json::{Value, json};

/// Creates a structured JSON error string.
///
/// # Arguments
/// * `message` - The primary error message.
/// * `code` - An optional error code string (e.g., "ENOENT", "EBADARG").
///   Defaults to "EUNKNOWN_RPC_ERROR".
///
/// # Returns
/// A JSON string representing the error.
pub fn rpc_error_string(message:String, code:Option<&str>) -> String {
	// Keep: Log the error being created, can be helpful for seeing what errors are
	// being generated Potentially only log if it's a severe code, or make logging
	// conditional. For now, let's assume this utility is called when an error is
	// definitive.
	if code.unwrap_or("").starts_with('E') && code.unwrap_or("") != "SUCCESS" {
		// Avoid logging success "errors"
		error!("[RPC Error Created] Code: {:?}, Message: {}", code.unwrap_or("N/A"), message);
	}

	json!({ "message": message, "code": code.unwrap_or("EUNKNOWN_RPC_ERROR") }).to_string()
}

/// Creates a structured JSON error string specifically for parameter validation
/// errors. Logs the error as well. The error code is fixed to "EBADARG".
///
/// # Arguments
/// * `method_name` - The name of the method or command where the error
///   occurred.
/// * `param_name` - The name of the problematic parameter.
/// * `expected_type` - A description of the expected type or format.
/// * `idx` - Optional index of the parameter if it's in an array.
///
/// # Returns
/// A JSON string representing the parameter error.
pub fn rpc_param_error_string(method_name:&str, param_name:&str, expected_type:&str, idx:Option<usize>) -> String {
	let base_msg = format!(
		"Missing or invalid '{}' parameter (expected {}) for method/command '{}'",
		param_name, expected_type, method_name
	);

	let full_msg = if let Some(i) = idx {
		format!("{} at arg index {}.", base_msg, i)
	} else {
		base_msg
	};

	// Note: `rpc_error_string` will also log this.
	rpc_error_string(full_msg, Some("EBADARG"))
}

/// Maps a `CommonError` from the effect system or other operations
/// to a structured JSON error string suitable for RPC/Tauri responses.
/// Logs the original error along with the context of the operation.
///
/// # Arguments
/// * `e` - The `CommonError` instance.
/// * `operation_context` - A string describing the operation during which the
///   error occurred (for logging).
///
/// # Returns
/// A JSON string representing the mapped error.
pub fn map_common_error_to_rpc_string(e:CommonError, operation_context:&str) -> String {
	// The `rpc_error_string` will log the mapped error. Logging the original
	// `CommonError` here provides context before it's transformed.
	error!(
		"[CommonError Mapping] Operation '{}' resulted in CommonError: {:?}",
		operation_context, e
	);

	let (message, code_str) = match e {
		CommonError::FsNotFound(p) => (format!("Resource not found: {}", p.display()), "ENOENT"),
		CommonError::FsPermissionDenied(p, m) => (format!("Permission denied for '{}': {}", p.display(), m), "EACCES"),
		CommonError::FsFileExists(p) => (format!("Resource already exists: {}", p.display()), "EEXIST"),
		CommonError::FsNotADirectory(p) => (format!("Path is not a directory: {}", p.display()), "ENOTDIR"),
		CommonError::FsIsADirectory(p) => (format!("Path is a directory: {}", p.display()), "EISDIR"),
		CommonError::FsNotEmpty(p) => (format!("Directory not empty: {}", p.display()), "ENOTEMPTY"),
		CommonError::FsRead(p, m) => (format!("Read error for '{}': {}", p.display(), m), "EIO_READ"),
		CommonError::FsWrite(p, m) => (format!("Write error for '{}': {}", p.display(), m), "EIO_WRITE"),
		CommonError::FsStat(p, m) => (format!("Stat error for '{}': {}", p.display(), m), "EIO_STAT"),
		CommonError::FsReadDir(p, m) => (format!("ReadDir error for '{}': {}", p.display(), m), "EIO_READDIR"),
		CommonError::FsMkdir(p, m) => (format!("Mkdir error for '{}': {}", p.display(), m), "EIO_MKDIR"),
		CommonError::FsDelete(p, m) => (format!("Delete error for '{}': {}", p.display(), m), "EIO_DELETE"),
		CommonError::FsRename(p, m) => (format!("Rename error for '{}': {}", p.display(), m), "EIO_RENAME"),
		CommonError::FsCopy(p, m) => (format!("Copy error for '{}': {}", p.display(), m), "EIO_COPY"),
		CommonError::ConfigUpdate(op, m) => {
			(format!("Configuration update error for '{}': {}", op, m), "ECONFIGUPDATE")
		},
		// Message from ConfigLoad is often sufficient
		CommonError::ConfigLoad(m) => (m, "ECONFIGLOAD"),
		CommonError::InvalidArg(arg_name, m) => (format!("Invalid argument '{}': {}", arg_name, m), "EBADARG"),
		CommonError::NotImplemented(feat) => (format!("Feature not implemented: {}", feat), "ENOSYS"),
		CommonError::StateLock(m) => (format!("Internal state access error: {}", m), "ESTATELOCK"),
		CommonError::IpcError(m) => (format!("Inter-process communication error: {}", m), "EIPC"),
		CommonError::CommandExecution(cmd, msg) => (format!("Command '{}' execution failed: {}", cmd, msg), "ECMDEXEC"),
		CommonError::CommandRegistration(cmd, msg) => {
			(format!("Command '{}' registration failed: {}", cmd, msg), "ECMDREG")
		},
		CommonError::CommandList(msg) => (format!("Failed to list commands: {}", msg), "ECMDLIST"),
		CommonError::SecretsAccess(key, msg) => (format!("Secret access for key '{}' failed: {}", key, msg), "ESECRET"),
		CommonError::OutputChannel(name, msg) => {
			(format!("Output channel '{}' operation failed: {}", name, msg), "EOUTPUT")
		},
		CommonError::Diagnostics(msg) => (format!("Diagnostics operation failed: {}", msg), "EDIAG"),
		CommonError::UiInteraction(msg) => (format!("UI interaction failed: {}", msg), "EUI"),
		// More specific general internal error
		CommonError::Unknown(m) => (m, "EUNKNOWN_INTERNAL_ERROR"),
	};

	rpc_error_string(message, Some(code_str))
}
