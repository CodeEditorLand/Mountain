// ---------------------------------------------------------------------------------------------
// Mountain Workspace FS API Handlers (handlers/workspace_fs_api.rs)
// --------------------------------------------------------------------------------------------
// Implements the backend logic for the `vscode.workspace.fs` filesystem API
// used by extensions. These handlers are invoked via RPC calls proxied from
// Cocoon's `fs-api-shim.js` through Vine and then routed by `track.rs` or
// `rpc.rs` to these specific functions.
//
// Responsibilities:
// - Handling specific `workspacefs_*` methods (e.g., `workspacefs_stat`,

//   `workspacefs_readFile`). These method names are conventions established by
//   `track.rs` or `rpc.rs` when dispatching.
// - Parsing URI components and operation-specific options from the RPC request
//   parameters (which are typically a `serde_json::Value` array).
// - Performing security checks, primarily URI scheme validation, to ensure that
//   requested paths are for supported schemes (e.g., 'file') and rejecting
//   unsupported ones (e.g., 'vscode-webview', 'http'). Path canonicalization
//   and workspace boundary checks are delegated to the `FsReader`/`FsWriter`
//   implementations in `environment.rs`.
// - Executing the underlying filesystem operations by dispatching to the
//   corresponding `FsReader` or `FsWriter` trait methods. These traits are
//   provided by the `MountainEnvironment` instance, accessed via the
//   `AppRuntime`.
// - Handling file types correctly when necessary (e.g., distinguishing files
//   from directories for operations like `delete`).
// - Formatting results into the JSON structures expected by Cocoon's
//   `fs-api-shim.js`. This includes:
//   - Base64 encoding for `readFile` results.
//   - `FileSystemStat` DTO structure for `stat` results.
//   - Directory listing format `[name: string, type: FileType][]` for
//     `readDirectory`.
// - Mapping `CommonError` instances (returned by `FsReader`/`FsWriter`) to
//   structured JSON-RPC error strings with appropriate error codes (e.g.,

//   "ENOENT", "EACCES") that the `fs-api-shim.js` can understand and convert
//   into `vscode.FileSystemError`.
//
// Key Interactions:
// - Called by `track.rs` (via direct function call after mapping the RPC method
//   name) or `rpc.rs` (if using `MainThreadFileSystemApiHandler` struct
//   methods).
// - Parses `Value` array parameters containing URI components and options.
// - Validates URI schemes to ensure they are supported for filesystem
//   operations.
// - Interacts with `AppRuntime` to get the `MountainEnvironment` instance.
// - Uses `env.require::<Arc<dyn FsReader/FsWriter>>()` to obtain the filesystem
//   providers.
// - Returns `Result<Value, String>` where the `Ok(Value)` is the JSON-formatted
//   success response and `Err(String)` is a JSON-formatted error string.
// --------------------------------------------------------------------------------------------

use std::{path::PathBuf, sync::Arc};

// Import necessary components from Land_Common
use Land_Common::{
	// Environment trait and Requires helper
	environment::{Environment, Requires},

	// CommonError enum for mapping
	errors::CommonError,

	// Filesystem traits and types
	fs_effects::{FileSystemStat, FileType as CommonFileType, FsReader, FsWriter},
};
// `futures::stream::TryStreamExt` is not needed if FsReader/FsWriter handles streams internally.
// Logging
use log::debug;
// For JSON manipulation
use serde_json::{Value, json};

// `url::Url` is not directly used here as path_from_uri_components_for_fs_api primarily extracts path string.
use crate::{
	// `AppState` might be needed for advanced context/permissions checks in the future,

	// but for now, path validation is delegated to environment.rs.
	// app_state::AppState,

	// For consistent error string creation
	handlers::error_utils,

	runtime::AppRuntime, /* AppRuntime required to access the Environment
	                      * `vine` might be needed for future FS event notifications to sidecars. */
};

// --- Helper Functions ---

// `create_handler_error_string` and `map_common_error_to_handler_string` are
// now centralized in `error_utils.rs`. This module will use
// `error_utils::rpc_error_string` and
// `error_utils::map_common_error_to_rpc_string`.

/// Helper to extract a `PathBuf` from `UriComponents` JSON `Value` for FS API
/// operations.
///
/// This function specifically validates that the URI scheme is 'file' (or
/// empty, implying 'file' for local paths) and rejects schemes known to be
/// unsupported by a typical local filesystem API (e.g., 'http',
///
///
/// 'vscode-remote').
///
/// # Argument
/// * `uri_val` - A `&serde_json::Value` expected to be an object representing
///   `UriComponents` (e.g., `{ "scheme": "file", "path": "/foo/bar.txt" }`).
///
/// # Returns
/// * `Ok(PathBuf)` if the URI is valid and a path can be extracted.
/// * `Err(String)` containing a JSON-RPC error string if the URI scheme is
///   unsupported, or if the 'path' field is missing or invalid.
fn path_from_uri_components_for_fs_api(uri_val:&Value) -> Result<PathBuf, String> {
	// Default to "file" scheme if not explicitly provided in the DTO.
	// Extensions using `vscode.workspace.fs` almost always operate on 'file' URIs
	// or Uris that can be resolved to file paths by a provider.
	// Assume "file" if scheme is missing
	let scheme = uri_val.get("scheme").and_then(Value::as_str).unwrap_or("file");

	match scheme {
		"file" | "" => {
			// "" scheme can occur if VS Code passes a string path that gets auto-converted
			// to URI without scheme.
			let path_str = uri_val.get("path").and_then(Value::as_str).ok_or_else(|| {
				error_utils::rpc_error_string(
					"Missing or invalid 'path' field in URI components for FS API operation.".to_string(),
					// Error Bad Argument for Path
					Some("EBADARG_PATH"),
				)
			})?;

			// TODO: Security Check - Further validation of `path_str` (e.g., ensuring it's
			//       not trying to escape a sandbox, if applicable) might be needed here or
			//       more robustly within the `FsReader`/`FsWriter` implementations in
			//       `environment.rs` which perform canonicalization and workspace boundary
			// checks.
			Ok(PathBuf::from(path_str))
		},

		// Explicitly reject schemes known to be unsupported by a standard local FS API.
		"vscode-webview" | "vscode-remote" | "vscode-resource" | "untitled" | "git" | "http" | "https" => {
			Err(error_utils::rpc_error_string(
				format!(
					"Unsupported URI scheme ('{}') for vscode.workspace.fs operations. Only 'file' scheme is \
					 typically supported by local FS providers.",
					scheme
				),
				// Error Not Supported for Scheme
				Some("ENOTSUP_SCHEME"),
			))
		},

		// Reject other unknown non-file schemes.
		_ => {
			Err(error_utils::rpc_error_string(
				format!(
					"WorkspaceFS API currently only supports 'file' scheme, but received '{}'.",
					scheme
				),
				Some("ENOTSUP_SCHEME"),
			))
		},
	}
}

// --- RPC Handlers (Called by Track dispatcher or rpc.rs) ---
// These handlers implement the `vscode.workspace.fs` API methods.
// They receive `Arc<AppRuntime>` directly from the dispatcher (`track.rs` or
// `rpc.rs`) instead of `State<'_, Arc<AppRuntime>>` to simplify their
// signatures, as they are not Tauri commands themselves but are called by them.

/// Handles the `workspacefs_stat` RPC call.
///
/// Corresponds to `vscode.workspace.fs.stat(uri)`.
///
/// # Argument
/// * `runtime` - The `AppRuntime` to access the `FsReader`.
/// * `params` - A `serde_json::Value` array: `[uri: UriComponents]`
///
/// # Returns
/// * `Ok(Value)`: JSON representation of `vscode.FileStat`.
/// * `Err(String)`: JSON-RPC error string.
pub async fn handle_workspace_fs_stat(
	runtime:Arc<AppRuntime>,

	// Expects Value::Array([uri_components_dto])
	params:Value,
) -> Result<Value, String> {
	let uri_components_dto = params.get(0).ok_or_else(|| {
		error_utils::rpc_param_error_string("workspacefs_stat", "uriComponents DTO", "Value::Object", Some(0))
	})?;

	let path = path_from_uri_components_for_fs_api(uri_components_dto)?;

	debug!("[WorkspaceFS Handler] Stat request for path: {}", path.display());

	let environment = runtime.get_environment();

	let fs_reader:Arc<dyn FsReader + Send + Sync> = environment.require();

	fs_reader
		.stat_file(&path)
		.await
		.map(|stat_obj:FileSystemStat| {
			// Convert the FileSystemStat (from Common) to the JSON Value format
			// expected by the vscode.workspace.fs API.
			json!({

				 // CommonFileType enum (u8)
				"type": stat_obj.file_type,


				 // Creation time in milliseconds since UNIX epoch
				"ctime": stat_obj.ctime,


				 // Modification time in milliseconds
				"mtime": stat_obj.mtime,


				 // Size in bytes
				"size": stat_obj.size,


				// `permissions` is optional in vscode.FileStat and CommonFileType.
				// Only include if Some.
				"permissions": stat_obj.permissions.map_or(Value::Null, |p| json!(p))
			})
		})
		.map_err(|common_err| error_utils::map_common_error_to_rpc_string(common_err, "vscode.workspace.fs.stat"))
}

/// Handles the `workspacefs_readDirectory` RPC call.
///
/// Corresponds to `vscode.workspace.fs.readDirectory(uri)`.
///
/// # Argument
/// * `runtime` - The `AppRuntime`.
/// * `params` - `[uri: UriComponents]`
///
/// # Returns
/// * `Ok(Value)`: JSON array of `[name: string, type: FileType][]`.
/// * `Err(String)`: JSON-RPC error string.
pub async fn handle_workspace_fs_read_directory(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let uri_components_dto = params.get(0).ok_or_else(|| {
		error_utils::rpc_param_error_string("workspacefs_readDirectory", "uriComponents DTO", "Value::Object", Some(0))
	})?;

	let path = path_from_uri_components_for_fs_api(uri_components_dto)?;

	debug!("[WorkspaceFS Handler] ReadDirectory request for path: {}", path.display());

	let environment = runtime.get_environment();

	let fs_reader:Arc<dyn FsReader + Send + Sync> = environment.require();

	fs_reader
		.read_directory(&path)
		.await
		.map(|entries_vec| {
			// Convert Vec<(String, CommonFileType)> to JSON Value array
			// CommonFileType (u8) directly matches vscode.FileType enum values.
			json!(entries_vec)
		})
		.map_err(|common_err| {
			error_utils::map_common_error_to_rpc_string(common_err, "vscode.workspace.fs.readDirectory")
		})
}

/// Handles the `workspacefs_readFile` RPC call.
///
/// Corresponds to `vscode.workspace.fs.readFile(uri)`.
///
/// # Argument
/// * `runtime` - The `AppRuntime`.
/// * `params` - `[uri: UriComponents]`
///
/// # Returns
/// * `Ok(Value)`: Base64 encoded string of file content. (Note: VS Code API
///   returns Uint8Array, shim expects base64 string from native for JSON)
/// * `Err(String)`: JSON-RPC error string.
pub async fn handle_workspace_fs_read_file(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let uri_components_dto = params.get(0).ok_or_else(|| {
		error_utils::rpc_param_error_string("workspacefs_readFile", "uriComponents DTO", "Value::Object", Some(0))
	})?;

	let path = path_from_uri_components_for_fs_api(uri_components_dto)?;

	debug!("[WorkspaceFS Handler] ReadFile request for path: {}", path.display());

	let environment = runtime.get_environment();

	let fs_reader:Arc<dyn FsReader + Send + Sync> = environment.require();

	match fs_reader.read_file(&path).await {
		Ok(bytes_vec) => {
			// Encode the byte vector as a base64 string for JSON transport.
			// The fs-api-shim.js in Cocoon will decode this back to a Uint8Array.
			let base64_content_str = base64::encode(&bytes_vec);

			// Return as JSON string
			Ok(json!(base64_content_str))
		},

		Err(common_err) => {
			Err(error_utils::map_common_error_to_rpc_string(
				common_err,
				"vscode.workspace.fs.readFile",
			))
		},
	}
}

/// Handles the `workspacefs_writeFile` RPC call.
///
/// Corresponds to `vscode.workspace.fs.writeFile(uri, content, options)`.
///
/// # Argument
/// * `runtime` - The `AppRuntime`.
/// * `params` - `[uri: UriComponents, content_base64: string, options?: {
///
///   create: boolean, overwrite: boolean }]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)`: JSON-RPC error string.
pub async fn handle_workspace_fs_write_file(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let params_array = params
		.as_array()
		.ok_or_else(|| error_utils::rpc_param_error_string("workspacefs_writeFile", "params", "array", None))?;

	let uri_components_dto = params_array.get(0).ok_or_else(|| {
		error_utils::rpc_param_error_string("workspacefs_writeFile", "uriComponents DTO", "Value::Object", Some(0))
	})?;

	let content_base64_str = params_array.get(1).and_then(Value::as_str).ok_or_else(|| {
		error_utils::rpc_param_error_string("workspacefs_writeFile", "content_base64", "string", Some(1))
	})?;

	// Optional options object
	let options_val = params_array.get(2).cloned().unwrap_or(Value::Null);

	// VS Code API defaults: create=true, overwrite=false for writeFile
	let create_opt = options_val.get("create").and_then(Value::as_bool).unwrap_or(true);

	let overwrite_opt = options_val.get("overwrite").and_then(Value::as_bool).unwrap_or(false);

	let path = path_from_uri_components_for_fs_api(uri_components_dto)?;

	debug!(
		"[WorkspaceFS Handler] WriteFile request for path: {}, create={}, overwrite={}",
		path.display(),
		create_opt,
		overwrite_opt
	);

	// Decode base64 content from string to Vec<u8>.
	let bytes_to_write = base64::decode(content_base64_str).map_err(|e| {
		error_utils::rpc_error_string(
			format!("Invalid base64 content provided for writeFile: {}", e),
			// Error Bad Message for Base64
			Some("EBADMSG_BASE64"),
		)
	})?;

	let environment = runtime.get_environment();

	let fs_writer:Arc<dyn FsWriter + Send + Sync> = environment.require();

	fs_writer
		.write_file(&path, bytes_to_write, create_opt, overwrite_opt)
		.await
		 // writeFile is void on success
		.map(|_| Value::Null)
		.map_err(|common_err| {

			error_utils::map_common_error_to_rpc_string(
				common_err,


				"vscode.workspace.fs.writeFile",


			)
		})
}

/// Handles the `workspacefs_createDirectory` RPC call.
///
/// Corresponds to `vscode.workspace.fs.createDirectory(uri)`. This operation is
/// recursive by VS Code API definition (creates parent directories if needed).
///
/// # Argument
/// * `runtime` - The `AppRuntime`.
/// * `params` - `[uri: UriComponents]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)`: JSON-RPC error string.
pub async fn handle_workspace_fs_create_directory(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let uri_components_dto = params.get(0).ok_or_else(|| {
		error_utils::rpc_param_error_string(
			"workspacefs_createDirectory",
			"uriComponents DTO",
			"Value::Object",
			Some(0),
		)
	})?;

	let path = path_from_uri_components_for_fs_api(uri_components_dto)?;

	debug!("[WorkspaceFS Handler] CreateDirectory request for path: {}", path.display());

	let environment = runtime.get_environment();

	let fs_writer:Arc<dyn FsWriter + Send + Sync> = environment.require();

	// `vscode.workspace.fs.createDirectory` is implicitly recursive.
	fs_writer
		 // `recursive` is true
		.create_directory(&path, true)
		.await
		 // createDirectory is void on success
		.map(|_| Value::Null)
		.map_err(|common_err| {

			error_utils::map_common_error_to_rpc_string(
				common_err,


				"vscode.workspace.fs.createDirectory",


			)
		})
}

/// Handles the `workspacefs_delete` RPC call.
///
/// Corresponds to `vscode.workspace.fs.delete(uri, options)`.
///
/// # Argument
/// * `runtime` - The `AppRuntime`.
/// * `params` - `[uri: UriComponents, options?: { recursive: bool, useTrash:
///   bool }]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)`: JSON-RPC error string.
pub async fn handle_workspace_fs_delete(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let params_array = params
		.as_array()
		.ok_or_else(|| error_utils::rpc_param_error_string("workspacefs_delete", "params", "array", None))?;

	let uri_components_dto = params_array.get(0).ok_or_else(|| {
		error_utils::rpc_param_error_string("workspacefs_delete", "uriComponents DTO", "Value::Object", Some(0))
	})?;

	let options_val = params_array.get(1).cloned().unwrap_or(Value::Null);

	// VS Code API defaults: recursive=false, useTrash=false
	let recursive_opt = options_val.get("recursive").and_then(Value::as_bool).unwrap_or(false);

	let use_trash_opt = options_val.get("useTrash").and_then(Value::as_bool).unwrap_or(false);

	let path = path_from_uri_components_for_fs_api(uri_components_dto)?;

	debug!(
		"[WorkspaceFS Handler] Delete request for path: {}, recursive={}, useTrash={}",
		path.display(),
		recursive_opt,
		use_trash_opt
	);

	if use_trash_opt {
		// Log if useTrash is requested but not implemented, then proceed with permanent
		// delete.
		warn!(
			"[WorkspaceFS Handler] 'useTrash=true' option for delete is requested but not fully implemented in MVP. \
			 Performing permanent delete."
		);

		// TODO: Implement `useTrash` functionality using a crate like `trash`
		// if desired.       If `useTrash` is critical and not implemented,

		// consider returning an error:       return
		// Err(error_utils::rpc_error_string("useTrash option not
		// implemented".to_string(), Some("ENOTSUP_TRASH")));
	}

	let environment = runtime.get_environment();

	let fs_writer:Arc<dyn FsWriter + Send + Sync> = environment.require();

	fs_writer
		 // Pass use_trash for future impl
		.delete(&path, recursive_opt, use_trash_opt)
		.await
		 // delete is void on success
		.map(|_| Value::Null)
		.map_err(|common_err| {

			error_utils::map_common_error_to_rpc_string(common_err, "vscode.workspace.fs.delete")
		})
}

/// Handles the `workspacefs_rename` RPC call (move operation).
///
/// Corresponds to `vscode.workspace.fs.rename(sourceUri, targetUri, options)`.
///
/// # Argument
/// * `runtime` - The `AppRuntime`.
/// * `params` - `[sourceUri: UriComponents, targetUri: UriComponents, options?:
///   { overwrite: bool }]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)`: JSON-RPC error string.
pub async fn handle_workspace_fs_rename(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let params_array = params
		.as_array()
		.ok_or_else(|| error_utils::rpc_param_error_string("workspacefs_rename", "params", "array", None))?;

	let source_uri_dto = params_array.get(0).ok_or_else(|| {
		error_utils::rpc_param_error_string("workspacefs_rename", "sourceUri DTO", "Value::Object", Some(0))
	})?;

	let target_uri_dto = params_array.get(1).ok_or_else(|| {
		error_utils::rpc_param_error_string("workspacefs_rename", "targetUri DTO", "Value::Object", Some(1))
	})?;

	let options_val = params_array.get(2).cloned().unwrap_or(Value::Null);

	// VS Code API default: overwrite=false
	let overwrite_opt = options_val.get("overwrite").and_then(Value::as_bool).unwrap_or(false);

	let source_path = path_from_uri_components_for_fs_api(source_uri_dto)?;

	let target_path = path_from_uri_components_for_fs_api(target_uri_dto)?;

	debug!(
		"[WorkspaceFS Handler] Rename request: {} -> {} (overwrite={})",
		source_path.display(),
		target_path.display(),
		overwrite_opt
	);

	let environment = runtime.get_environment();

	let fs_writer:Arc<dyn FsWriter + Send + Sync> = environment.require();

	fs_writer
		.rename(&source_path, &target_path, overwrite_opt)
		.await
		 // rename is void on success
		.map(|_| Value::Null)
		.map_err(|common_err| {

			error_utils::map_common_error_to_rpc_string(common_err, "vscode.workspace.fs.rename")
		})
}

/// Handles the `workspacefs_copy` RPC call.
///
/// Corresponds to `vscode.workspace.fs.copy(sourceUri, targetUri, options)`.
///
/// # Argument
/// * `runtime` - The `AppRuntime`.
/// * `params` - `[sourceUri: UriComponents, targetUri: UriComponents, options?:
///   { overwrite: bool }]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)`: JSON-RPC error string.
pub async fn handle_workspace_fs_copy(runtime:Arc<AppRuntime>, params:Value) -> Result<Value, String> {
	let params_array = params
		.as_array()
		.ok_or_else(|| error_utils::rpc_param_error_string("workspacefs_copy", "params", "array", None))?;

	let source_uri_dto = params_array.get(0).ok_or_else(|| {
		error_utils::rpc_param_error_string("workspacefs_copy", "sourceUri DTO", "Value::Object", Some(0))
	})?;

	let target_uri_dto = params_array.get(1).ok_or_else(|| {
		error_utils::rpc_param_error_string("workspacefs_copy", "targetUri DTO", "Value::Object", Some(1))
	})?;

	let options_val = params_array.get(2).cloned().unwrap_or(Value::Null);

	// VS Code API default: overwrite=false
	let overwrite_opt = options_val.get("overwrite").and_then(Value::as_bool).unwrap_or(false);

	let source_path = path_from_uri_components_for_fs_api(source_uri_dto)?;

	let target_path = path_from_uri_components_for_fs_api(target_uri_dto)?;

	debug!(
		"[WorkspaceFS Handler] Copy request: {} -> {} (overwrite={})",
		source_path.display(),
		target_path.display(),
		overwrite_opt
	);

	let environment = runtime.get_environment();

	// Copy operation might require both read (from source) and write (to target)
	// capabilities. The FsWriter trait is expected to handle this.
	let fs_writer:Arc<dyn FsWriter + Send + Sync> = environment.require();

	// `FsWriter::copy` should handle recursive copy if the source is a directory.
	fs_writer
		.copy(&source_path, &target_path, overwrite_opt)
		.await
		 // copy is void on success
		.map(|_| Value::Null)
		.map_err(|common_err| {

			error_utils::map_common_error_to_rpc_string(common_err, "vscode.workspace.fs.copy")
		})
}
