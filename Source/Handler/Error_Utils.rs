// ---------------------------------------------------------------------------------------------
// Mountain Error Utilities 
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
// No `Value` import needed as we directly create JSON strings.
use log::error;
use serde_json::json;

/// Creates a structured JSON error string suitable for RPC responses.
///
/// The resulting JSON string will have a "message" field and a "code" field.
/// Errors created through this function are logged internally.
///
/// # Argument
/// * `message` - The primary human-readable error message.
/// * `code` - An optional error code string (e.g., "ENOENT", "EBADARG").
///   Defaults to "EUNKNOWN_RPC_ERROR" if `None`.
///
/// # Returns
/// A `String` containing the JSON representation of the error.
pub fn rpc_error_string(message:String, code:Option<&str>) -> String {
	// Log the error being created. This is crucial for server-side diagnostics.
	// Avoid logging if the "code" indicates a success-like scenario that might
	// be (mis)using this error utility.
	let error_code = code.unwrap_or("EUNKNOWN_RPC_ERROR");

	if !(error_code.eq_ignore_ascii_case("SUCCESS") || error_code.is_empty()) {
		error!("[RPC Error Gen] Code: '{}', Message: '{}'", error_code, message);
	}

	json!({


		"message": message,


		"code": error_code
	})
	.to_string()
}

/// Creates a structured JSON error string specifically for parameter validation
/// errors.
///
/// This is a convenience wrapper around `rpc_error_string` that standardizes
/// the message format and uses the "EBADARG" error code. The error is also
/// logged.
///
/// # Argument
/// * `method_name` - The name of the method or command where the error
///   occurred.
/// * `param_name` - The name of the problematic parameter.
/// * `expected_type` - A description of the expected type or format of the
///   parameter.
/// * `idx` - Optional index of the parameter if it's part of an array of
///   arguments.
///
/// # Returns
/// A `String` containing the JSON representation of the parameter error.
pub fn rpc_param_error_string(method_name:&str, param_name:&str, expected_type:&str, idx:Option<usize>) -> String {
	let base_msg = format!(
		"Missing or invalid parameter '{}' (expected {}) for method/command '{}'",
		param_name, expected_type, method_name
	);

	let full_msg = if let Some(i) = idx {
		format!("{} at argument index {}.", base_msg, i)
	} else {
		base_msg
	};

	// `rpc_error_string` will log this error with "EBADARG" code.
	rpc_error_string(full_msg, Some("EBADARG"))
}

/// Maps a `CommonError` (typically from the effect system or internal
/// operations) to a structured JSON error string suitable for RPC/Tauri
/// responses.
///
/// This function ensures that internal errors are translated into a consistent,
///
///
/// user-facing format with appropriate error codes. The original `CommonError`
/// is logged for detailed internal diagnostics.
///
/// # Argument
/// * `e` - The `CommonError` instance to map.
/// * `operation_context` - A string describing the operation during which the
///   error occurred (e.g., "native_save_all", "fs.readFile"). This is used for
///   logging clarity.
///
/// # Returns
/// A `String` containing the JSON representation of the mapped error.
pub fn map_common_error_to_rpc_string(e:CommonError, operation_context:&str) -> String {
	// Log the original CommonError with its context before transformation.
	// This provides crucial debugging information.
	error!(
		"[CommonError Mapping] Operation '{}' resulted in CommonError: {:?}",
		operation_context, e
	);

	// TODO: Review error codes (the string part, e.g., "ENOENT") to align with
	//       standard POSIX codes where applicable, or VS Code's internal error
	//       codes for better client-side handling if the client expects specific
	//       codes.
	let (message, code_str) = match e {
		// Filesystem Errors
		CommonError::FsNotFound(p) => {
			(
				format!("Resource not found: {}", p.display()),
				// Standard POSIX: No such file or directory
				"ENOENT",
			)
		},

		CommonError::FsPermissionDenied(p, m) => {
			(
				format!("Permission denied for '{}': {}", p.display(), m),
				// Standard POSIX: Permission denied
				"EACCES",
			)
		},

		CommonError::FsFileExists(p) => {
			(
				format!("Resource already exists: {}", p.display()),
				// Standard POSIX: File exists
				"EEXIST",
			)
		},

		CommonError::FsNotADirectory(p) => {
			(
				format!("Path is not a directory: {}", p.display()),
				// Standard POSIX: Not a directory
				"ENOTDIR",
			)
		},

		CommonError::FsIsADirectory(p) => {
			(
				format!("Path is a directory: {}", p.display()),
				// Standard POSIX: Is a directory
				"EISDIR",
			)
		},

		CommonError::FsNotEmpty(p) => {
			(
				format!("Directory not empty: {}", p.display()),
				// Standard POSIX: Directory not empty
				"ENOTEMPTY",
			)
		},

		// Generic I/O errors for FS operations
		CommonError::FsRead(p, m) => {
			(
				format!("Read error for '{}': {}", p.display(), m),
				// Custom: I/O error during read
				"EIO_READ",
			)
		},

		CommonError::FsWrite(p, m) => {
			(
				format!("Write error for '{}': {}", p.display(), m),
				// Custom: I/O error during write
				"EIO_WRITE",
			)
		},

		CommonError::FsStat(p, m) => {
			(
				format!("Stat error for '{}': {}", p.display(), m),
				// Custom: I/O error during stat
				"EIO_STAT",
			)
		},

		CommonError::FsReadDir(p, m) => {
			(
				format!("ReadDir error for '{}': {}", p.display(), m),
				// Custom: I/O error during readdir
				"EIO_READDIR",
			)
		},

		CommonError::FsMkdir(p, m) => {
			(
				format!("Mkdir error for '{}': {}", p.display(), m),
				// Custom: I/O error during mkdir
				"EIO_MKDIR",
			)
		},

		CommonError::FsDelete(p, m) => {
			(
				format!("Delete error for '{}': {}", p.display(), m),
				// Custom: I/O error during delete
				"EIO_DELETE",
			)
		},

		CommonError::FsRename(p, m) => {
			(
				format!("Rename error for '{}': {}", p.display(), m),
				// Custom: I/O error during rename
				"EIO_RENAME",
			)
		},

		CommonError::FsCopy(p, m) => {
			(
				format!("Copy error for '{}': {}", p.display(), m),
				// Custom: I/O error during copy
				"EIO_COPY",
			)
		},

		// Configuration Errors
		CommonError::ConfigUpdate(op, m) => {
			(
				format!("Configuration update error for '{}': {}", op, m),
				// Custom: Configuration update error
				"ECONFIGUPDATE",
			)
		},

		CommonError::ConfigLoad(m) => {
			(
				// Message from ConfigLoad is often sufficient and detailed
				m,
				// Custom: Configuration load error
				"ECONFIGLOAD",
			)
		},

		// General Application Errors
		CommonError::InvalidArg(arg_name, m) => {
			(
				format!("Invalid argument '{}': {}", arg_name, m),
				// Matches the code used in rpc_param_error_string
				"EBADARG",
			)
		},

		CommonError::NotImplemented(feat) => {
			(
				format!("Feature not implemented: {}", feat),
				// Standard POSIX: Function not implemented
				"ENOSYS",
			)
		},

		CommonError::StateLock(m) => {
			(
				format!("Internal state access error: {}", m),
				// Custom: State lock error
				"ESTATELOCK",
			)
		},

		CommonError::IpcError(m) => {
			(
				format!("Inter-process communication error: {}", m),
				// Custom: IPC error
				"EIPC",
			)
		},

		// Command System Errors
		CommonError::CommandExecution(cmd, msg) => {
			(
				format!("Command '{}' execution failed: {}", cmd, msg),
				// Custom: Command execution error
				"ECMDEXEC",
			)
		},

		CommonError::CommandRegistration(cmd, msg) => {
			(
				format!("Command '{}' registration failed: {}", cmd, msg),
				// Custom: Command registration error
				"ECMDREG",
			)
		},

		CommonError::CommandList(msg) => {
			(
				format!("Failed to list commands: {}", msg),
				// Custom: Command list error
				"ECMDLIST",
			)
		},

		// Other Specific Errors
		CommonError::SecretsAccess(key, msg) => {
			(
				format!("Secret access for key '{}' failed: {}", key, msg),
				// Custom: Secret access error
				"ESECRET",
			)
		},

		CommonError::OutputChannel(name, msg) => {
			(
				format!("Output channel '{}' operation failed: {}", name, msg),
				// Custom: Output channel error
				"EOUTPUT",
			)
		},

		CommonError::Diagnostics(msg) => {
			(
				format!("Diagnostics operation failed: {}", msg),
				// Custom: Diagnostics error
				"EDIAG",
			)
		},

		CommonError::UiInteraction(msg) => {
			(
				format!("UI interaction failed: {}", msg),
				// Custom: UI interaction error
				"EUI",
			)
		},

		// Catch-all Unknown Error
		CommonError::Unknown(m) => {
			(
				// Use the message directly as it's already formatted
				m,
				// More specific than just EUNKNOWN_RPC_ERROR
				"EUNKNOWN_INTERNAL",
			)
		},
	};

	// `rpc_error_string` will log the final mapped error as well.
	rpc_error_string(message, Some(code_str))
}
