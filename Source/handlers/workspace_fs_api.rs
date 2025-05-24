// ---------------------------------------------------------------------------------------------
// Mountain Workspace FS API Handlers (handlers/workspace_fs_api.rs)
// --------------------------------------------------------------------------------------------
// Implements the backend logic for the `vscode.workspace.fs` filesystem API
// used by extensions. These handlers are invoked via RPC calls proxied from
// Cocoon's `fs-api-shim.js` through Vine and Track.
//
// Responsibilities:
// - Handling specific `workspacefs_*` methods (`stat`, `readFile`, `writeFile`,

//   `readDirectory`, `createDirectory`, `delete`, `rename`, `copy`).
// - Parsing URI components and options from the RPC request parameters (Value
//   array).
// - Performing security checks (scheme validation) to ensure requested paths
//   are valid.
// - Executing the underlying filesystem operations by dispatching to the
//   corresponding `FsReader`/`FsWriter` trait methods provided by the
//   `Environment` obtained from the `AppRuntime`.
// - Handling file types correctly (e.g., distinguishing files/directories for
//   `delete`).
// - Formatting results (e.g., base64 for `readFile`, stat structure, directory
//   listing format) and errors (mapping CommonError to structured JSON errors
//   with codes like `ENOENT`) as expected by the `fs-api-shim.js`.
// - Rejecting requests for unsupported URI schemes (like `vscode-webview`,

//   `vscode-remote`).
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` (mapped from `workspacefs_*`
//   methods).
// - Parses `Value` array params containing URI components and options.
// - Validates URI schemes.
// - Interacts with `AppRuntime` to get the `Environment` and `require` the
//   `FsReader`/`FsWriter`.
// - Returns `Result<Value, String>` where the error string is structured JSON.
// --------------------------------------------------------------------------------------------

use std::{path::PathBuf, sync::Arc};

// Import necessary components from Land_Common
use Land_Common::{
	// Environment trait and Require helper
	environment::{Environment, Requires},

	// Error enum for mapping
	errors::CommonError,

	// Filesystem traits
	fs_effects::{FsReader, FsWriter},
};
// Not needed if FsReader handles stream internally
// use futures::stream::TryStreamExt;

// Use log crate
use log;
use serde_json::{Value, json};
// Tauri imports
use tauri::{AppHandle, Manager, Runtime as TauriRuntime, State, Window};

// Url crate not directly needed if only dealing with paths
// use url::Url;
use crate::{
	// AppState might be needed for context/permissions later
	app_state::AppState,

	runtime::AppRuntime,
	// Vine might be needed for future FS event notifications
	// Runtime required to access Environment
	// use crate::vine;
};

// --- Helper Functions ---

/// Creates a structured error JSON string for RPC error responses.
fn create_handler_error_string(message:String, code:Option<&str>) -> String {
	json!({


		 "message": message,

		  // Default error code
		 "code": code.unwrap_or("EUNKNOWN")
	})
	.to_string()
}

/// Maps `CommonError` variants (typically returned by FsReader/FsWriter)
/// to structured error JSON strings expected by the fs-api-shim.
fn map_common_error_to_handler_string(e:CommonError) -> String {
	let (message, code) = match e {
		CommonError::FsNotFound(p) => (format!("File not found: {}", p.display()), Some("ENOENT")),

		CommonError::FsPermissionDenied(p, m) => {
			(format!("Permission denied for '{}': {}", p.display(), m), Some("EACCES"))
		},

		CommonError::FsFileExists(p) => (format!("File already exists: {}", p.display()), Some("EEXIST")),

		CommonError::FsNotADirectory(p) => (format!("Path is not a directory: {}", p.display()), Some("ENOTDIR")),

		CommonError::FsIsADirectory(p) => (format!("Path is a directory: {}", p.display()), Some("EISDIR")),

		CommonError::FsNotEmpty(p) => (format!("Directory not empty: {}", p.display()), Some("ENOTEMPTY")),

		// Map generic IO errors
		CommonError::FsRead(p, m) => (format!("Read failed for '{}': {}", p.display(), m), Some("EIO")),

		CommonError::FsWrite(p, m) => (format!("Write failed for '{}': {}", p.display(), m), Some("EIO")),

		CommonError::FsStat(p, m) => (format!("Stat failed for '{}': {}", p.display(), m), Some("EIO")),

		CommonError::FsReadDir(p, m) => (format!("ReadDir failed for '{}': {}", p.display(), m), Some("EIO")),

		CommonError::FsMkdir(p, m) => (format!("Mkdir failed for '{}': {}", p.display(), m), Some("EIO")),

		CommonError::FsDelete(p, m) => (format!("Delete failed for '{}': {}", p.display(), m), Some("EIO")),

		CommonError::FsRename(p, m) => (format!("Rename failed for '{}': {}", p.display(), m), Some("EIO")),

		CommonError::FsCopy(p, m) => (format!("Copy failed for '{}': {}", p.display(), m), Some("EIO")),

		// Map argument/logic errors
		CommonError::InvalidArg(a, m) => (format!("Invalid argument '{}': {}", a, m), Some("EINVAL")), /* Invalid Argument code */
		CommonError::NotImplemented(f) => (format!("Operation not implemented: {}", f), Some("ENOSYS")), /* Not Implemented code */
		// Default fallback
		// Unknown error
		_ => (e.to_string(), Some("EUNKNOWN")),
	};

	create_handler_error_string(message, code)
}

/// Helper to get a PathBuf from URI components JSON Value received via RPC.
/// Enforces 'file' scheme and rejects unsupported schemes.
fn path_from_uri_components(uri_val:&Value) -> Result<PathBuf, String> {
	// Default to file scheme if missing
	let scheme = uri_val.get("scheme").and_then(|v| v.as_str()).unwrap_or("file");

	match scheme {
		// Allow 'file' scheme or empty scheme (could imply relative paths, but fs effects likely expect absolute)
		"file" | "" => {
			let path_str = uri_val.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
				create_handler_error_string("Missing or invalid 'path' in URI components".to_string(), Some("EBADARG"))
			})?;

			// TODO: Security check - Ensure path is within allowed workspace boundaries?
			// This might belong in the Environment impl.
			Ok(PathBuf::from(path_str))
		},

		// Explicitly reject schemes known to be unsupported by the standard FS API
		"vscode-webview" | "vscode-remote" | "vscode-resource" | "untitled" | "git" | "http" | "https" => {
			Err(create_handler_error_string(
				format!("Unsupported URI scheme for workspace.fs: {}", scheme),
				// Operation Not Supported code
				Some("ENOTSUP"),
			))
		},

		// Reject other non-file schemes
		_ => {
			Err(create_handler_error_string(
				format!("WorkspaceFS API currently only supports 'file' scheme, got '{}'", scheme),
				Some("ENOTSUP"),
			))
		},
	}
}

// --- RPC Handlers (Called by Track dispatcher) ---
// These handlers delegate the actual filesystem work to the appropriate trait
// (`FsReader` or `FsWriter`) obtained from the `AppRuntime`'s `Environment`.

/// Handles `$stat` RPC call.
/// Args: `[uri: UriComponents]`
pub async fn handle_stat(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let uri_val = params
		 // RPC calls use array params
		.get(0)
		.ok_or_else(|| create_handler_error_string("Missing 'uri' parameter".to_string(), Some("EBADARG")))?;

	// Validates scheme
	let path = path_from_uri_components(uri_val)?;

	log::debug!("[WorkspaceFS Handler] handle_stat: {}", path.display());

	// Get FsReader from runtime/environment and call its method
	let env = runtime.get_environment();

	let fs_reader:Arc<dyn FsReader + Send + Sync> = env.require();

	fs_reader
		 // FsReader::stat_file should return Result<Value, CommonError>
		.stat_file(&path)
		.await
		 // Map CommonError to JSON error string
		.map_err(map_common_error_to_handler_string)
}

/// Handles `$readDirectory` RPC call.
/// Args: `[uri: UriComponents]`
pub async fn handle_read_directory(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let uri_val = params
		.get(0)
		.ok_or_else(|| create_handler_error_string("Missing 'uri' parameter".to_string(), Some("EBADARG")))?;

	// Validates scheme
	let path = path_from_uri_components(uri_val)?;

	log::debug!("[WorkspaceFS Handler] handle_readDirectory: {}", path.display());

	let env = runtime.get_environment();

	let fs_reader:Arc<dyn FsReader + Send + Sync> = env.require();

	fs_reader
		 // FsReader::read_directory should return Result<Vec<(String, String)>, CommonError>
		.read_directory(&path)
		.await
		 // Convert Vec<(name, type)> to JSON Value array
		.map(|entries| json!(entries))
		.map_err(map_common_error_to_handler_string)
}

/// Handles `$readFile` RPC call.
/// Args: `[uri: UriComponents]`
pub async fn handle_read_file(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let uri_val = params
		.get(0)
		.ok_or_else(|| create_handler_error_string("Missing 'uri' parameter".to_string(), Some("EBADARG")))?;

	// Validates scheme
	let path = path_from_uri_components(uri_val)?;

	log::debug!("[WorkspaceFS Handler] handle_readFile: {}", path.display());

	let env = runtime.get_environment();

	let fs_reader:Arc<dyn FsReader + Send + Sync> = env.require();

	match fs_reader.read_file(&path).await {
		// FsReader::read_file should return Result<Vec<u8>, CommonError>
		Ok(bytes) => {
			// Encode the byte vector as base64 for JSON transport
			let base64_content = base64::encode(&bytes);

			// Return JSON string (base64 encoded)
			Ok(json!(base64_content))
		},

		Err(e) => Err(map_common_error_to_handler_string(e)),
	}
}

/// Handles `$writeFile` RPC call.
/// Args: `[uri: UriComponents, content_base64: string, options: { create: bool,
///
/// overwrite: bool }]`
pub async fn handle_write_file(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let uri_val = params
		.get(0)
		.ok_or_else(|| create_handler_error_string("Missing 'uri' parameter".to_string(), Some("EBADARG")))?;

	let content_b64 = params.get(1).and_then(Value::as_str).ok_or_else(|| {
		create_handler_error_string("Missing 'content' (base64 string) parameter".to_string(), Some("EBADARG"))
	})?;

	// TODO: Parse options from params[2] if the FsWriter::write_file method needs
	// them (e.g., create, overwrite). let options_val =
	// params.get(2).cloned().unwrap_or(Value::Null); let create =
	// options_val.get("create").and_then(Value::as_bool).unwrap_or(true); //
	// Default create=true? let overwrite =
	// options_val.get("overwrite").and_then(Value::as_bool).unwrap_or(true); //
	// Default overwrite=true?

	// Validates scheme
	let path = path_from_uri_components(uri_val)?;

	log::debug!("[WorkspaceFS Handler] handle_writeFile: {}", path.display());

	// Decode base64 content
	let bytes = base64::decode(content_b64)
		 // Bad Message code
		.map_err(|e| create_handler_error_string(format!("Invalid base64 content provided: {}", e), Some("EBADMSG")))?;

	let env = runtime.get_environment();

	let fs_writer:Arc<dyn FsWriter + Send + Sync> = env.require();

	// Call FsWriter::write_file, potentially passing create/overwrite options if
	// supported
	fs_writer
		.write_file(&path, bytes /*, create, overwrite */)
		.await
		 // Return JSON null on success
		.map(|_| Value::Null)
		.map_err(map_common_error_to_handler_string)
}

/// Handles `$createDirectory` RPC call.
/// Args: `[uri: UriComponents]`
pub async fn handle_create_directory(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let uri_val = params
		.get(0)
		.ok_or_else(|| create_handler_error_string("Missing 'uri' parameter".to_string(), Some("EBADARG")))?;

	// Validates scheme
	let path = path_from_uri_components(uri_val)?;

	log::debug!("[WorkspaceFS Handler] handle_createDirectory: {}", path.display());

	let env = runtime.get_environment();

	let fs_writer:Arc<dyn FsWriter + Send + Sync> = env.require();

	fs_writer
		 // FsWriter::create_directory handles intermediate dirs
		.create_directory(&path)
		.await
		.map(|_| Value::Null)
		.map_err(map_common_error_to_handler_string)
}

/// Handles `$delete` RPC call.
/// Args: `[uri: UriComponents, options: { recursive: bool, useTrash: bool }]`
pub async fn handle_delete(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let uri_val = params
		.get(0)
		.ok_or_else(|| create_handler_error_string("Missing 'uri' parameter".to_string(), Some("EBADARG")))?;

	// Options object
	let options_val = params.get(1).cloned().unwrap_or(Value::Null);

	let recursive = options_val.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);

	// TODO: Handle useTrash option if supported by FsWriter::delete
	let use_trash = options_val.get("useTrash").and_then(|v| v.as_bool()).unwrap_or(false);

	if use_trash {
		log::warn!("[WorkspaceFS Handler] 'useTrash' option for delete is not implemented, using permanent delete.");
	}

	// Validates scheme
	let path = path_from_uri_components(uri_val)?;

	log::debug!(
		"[WorkspaceFS Handler] handle_delete: {} (recursive={})",
		path.display(),
		recursive
	);

	let env = runtime.get_environment();

	let fs_writer:Arc<dyn FsWriter + Send + Sync> = env.require();

	fs_writer
		 // Pass options if FsWriter supports them
		.delete(&path, recursive /*, use_trash */)
		.await
		.map(|_| Value::Null)
		.map_err(map_common_error_to_handler_string)
}

/// Handles `$rename` RPC call (move).
/// Args: `[sourceUri: UriComponents, targetUri: UriComponents, options: {
///
///
/// overwrite: bool }]`
pub async fn handle_rename(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let source_uri_val = params
		.get(0)
		.ok_or_else(|| create_handler_error_string("Missing 'source' uri parameter".to_string(), Some("EBADARG")))?;

	let target_uri_val = params
		.get(1)
		.ok_or_else(|| create_handler_error_string("Missing 'target' uri parameter".to_string(), Some("EBADARG")))?;

	// Options object
	let options_val = params.get(2).cloned().unwrap_or(Value::Null);

	let overwrite = options_val.get("overwrite").and_then(|v| v.as_bool()).unwrap_or(false);

	// Validates scheme
	let source_path = path_from_uri_components(source_uri_val)?;

	// Validates scheme
	let target_path = path_from_uri_components(target_uri_val)?;

	log::debug!(
		"[WorkspaceFS Handler] handle_rename: {} -> {} (overwrite={})",
		source_path.display(),
		target_path.display(),
		overwrite
	);

	let env = runtime.get_environment();

	let fs_writer:Arc<dyn FsWriter + Send + Sync> = env.require();

	fs_writer
		.rename(&source_path, &target_path, overwrite)
		.await
		.map(|_| Value::Null)
		.map_err(map_common_error_to_handler_string)
}

/// Handles `$copy` RPC call.
/// Args: `[sourceUri: UriComponents, targetUri: UriComponents, options: {
///
///
/// overwrite: bool }]`
pub async fn handle_copy(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let source_uri_val = params
		.get(0)
		.ok_or_else(|| create_handler_error_string("Missing 'source' uri parameter".to_string(), Some("EBADARG")))?;

	let target_uri_val = params
		.get(1)
		.ok_or_else(|| create_handler_error_string("Missing 'target' uri parameter".to_string(), Some("EBADARG")))?;

	// Options object
	let options_val = params.get(2).cloned().unwrap_or(Value::Null);

	let overwrite = options_val.get("overwrite").and_then(|v| v.as_bool()).unwrap_or(false);

	// Validates scheme
	let source_path = path_from_uri_components(source_uri_val)?;

	// Validates scheme
	let target_path = path_from_uri_components(target_uri_val)?;

	log::debug!(
		"[WorkspaceFS Handler] handle_copy: {} -> {} (overwrite={})",
		source_path.display(),
		target_path.display(),
		overwrite
	);

	let env = runtime.get_environment();

	// Copy requires both read and write capabilities potentially
	let fs_writer:Arc<dyn FsWriter + Send + Sync> = env.require();

	// FsWriter::copy should handle recursive copy if source is directory
	fs_writer
		.copy(&source_path, &target_path, overwrite)
		.await
		.map(|_| Value::Null)
		.map_err(map_common_error_to_handler_string)
}
