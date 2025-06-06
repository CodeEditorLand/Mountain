// ---------------------------------------------------------------------------------------------
// Mountain Native FS Handlers  - DEPRECATED
// --------------------------------------------------------------------------------------------
// This file previously handled low-level filesystem requests proxied directly
// from Cocoon's Node 'fs' module shim (using `fs_*` method names like
// `fs_stat`, `fs_readFile`, etc.).
//
// **CURRENT STATUS: DEPRECATED AND UNUSED**
//
// With the implementation of the `vscode.workspace.fs` API shim
// (`fs-api-shim.js` in Cocoon) and its corresponding Mountain handlers
// (`handlers::workspace_fs_api.rs`), which delegate to the
// `FsReader`/`FsWriter` traits implemented by `MountainEnvironment`
// (`environment.rs`), direct proxying of the Node 'fs' module is no longer used
// or supported in the primary application flow.
//
// Extensions *must* use `vscode.workspace.fs` for filesystem operations.
// If direct Node 'fs' access were to be re-enabled (which is discouraged for
// security and abstraction reasons), these handlers would require significant
// updates, including robust security checks, path validation against workspace
// boundaries, and alignment with the Environment's FS capabilities.
//
// This file is kept **for historical reference only**. Its functions should
// **not** be called and will return errors indicating they are deprecated.
// --------------------------------------------------------------------------------------------

// ----- START: DEPRECATED Element/Mountain/src/Handler/native_fs.rs -----
// NOTE: This entire file is DEPRECATED.
//       Extensions should use `vscode.workspace.fs` API, which is handled by
//       `handlers::workspace_fs_api.rs` and `environment.rs`.

use std::path::PathBuf;

// These imports might not be strictly needed if the file is truly unused and
// functions are just stubs, but kept for context of what they might have used.
// Example of a hypothetical external crate
// use Land_River::api as river_api;

// Example of a hypothetical external crate
// use Land_Sun::api as sun_api;

// Use log crate for logging warnings about deprecation
use log;
use serde_json::{Value, json};
// Tauri imports for handler signatures
use tauri::Runtime;

/// Constant error message for deprecated native FS handlers.
const DEPRECATED_NATIVE_FS_ERROR_MSG:&str =
	"Native FS direct proxy handlers are deprecated; use the vscode.workspace.fs API via effects/environment.";

// --- Helper Functions (remain unchanged, but unused and part of deprecated
// code) ---

/// **DEPRECATED:** Creates a JSON error string.
fn create_deprecated_error_string(message:String, code:Option<&str>) -> String {
	json!({ "message": message, "code": code.unwrap_or("ENOSYS_DEPRECATED") }).to_string()
}

/// **DEPRECATED:** Helper to parse a PathBuf from URI components.
///
/// In a real implementation, this would parse `params` which would be a
/// `Value` containing URI components. Now, it immediately returns an error.
fn path_from_uri_components_deprecated(_params:&Value) -> Result<PathBuf, String> {
	Err(create_deprecated_error_string(
		DEPRECATED_NATIVE_FS_ERROR_MSG.to_string(),
		// Code already in DEPRECATED_NATIVE_FS_ERROR_MSG via create_deprecated_error_string
		None,
	))
}

// --- Direct Action Handlers (all deprecated stubs) ---
// These handlers would have been for more direct, Mountain-initiated FS actions
// if they weren't part of the Node 'fs' proxy.

/// **DEPRECATED:** Use `vscode.workspace.fs` API effects (e.g.,
///
/// `fs_effects::read_file`) instead.
///
/// This function is a stub and will always return an error.
// Expected signature parameters, now unused:
pub async fn handle_read_file_deprecated<R:Runtime>(// _app: AppHandle<R>,

	// _window: Window<R>,

	// _params: Value,
) -> Result<Value, String> {
	log::warn!("[FS Handler - DEPRECATED] Direct handle_read_file_deprecated called. This handler is non-functional.");

	Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.to_string(), None))
}

/// **DEPRECATED:** Use `vscode.workspace.fs` API effects (e.g.,
///
/// `fs_effects::write_file`) instead.
///
/// This function is a stub and will always return an error.
// Expected signature parameters, now unused.)
pub async fn handle_write_file_deprecated<R:Runtime>(// _app: AppHandle<R>,

	// _window: Window<R>,

	// _params: Value
) -> Result<Value, String> {
	log::warn!("[FS Handler - DEPRECATED] Direct handle_write_file_deprecated called. This handler is non-functional.");

	Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.to_string(), None))
}

// --- Proxied Node 'fs' Handlers (all deprecated stubs) ---
// These handlers were intended to mirror Node.js 'fs' module methods.

/// **DEPRECATED:** Use `vscode.workspace.fs.stat` (via
/// `handlers::workspace_fs_api::handle_stat`) instead.
///
/// This function is a stub and will always return an error.
pub async fn handle_fs_stat_deprecated(
	// Parameters from Cocoon's fs.stat call
	_params:Value,
) -> Result<Value, String> {
	log::warn!("[FS Handler Proxy - DEPRECATED] handle_fs_stat_deprecated called. This handler is non-functional.");

	Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.to_string(), None))
}

/// **DEPRECATED:** `realpath` functionality should be handled by path
/// canonicalization within `vscode.workspace.fs` implementations if needed, or
/// by the extension itself using path utilities.
///
/// This function is a stub and will always return an error.
pub async fn handle_fs_realpath_deprecated(
	// Parameters from Cocoon's fs.realpath call
	_params:Value,
) -> Result<Value, String> {
	log::warn!("[FS Handler Proxy - DEPRECATED] handle_fs_realpath_deprecated called. This handler is non-functional.");

	Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.to_string(), None))
}

/// **DEPRECATED:** Use `vscode.workspace.fs.readFile` (via
/// `handlers::workspace_fs_api::handle_read_file`) instead.
///
/// This function is a stub and will always return an error.
pub async fn handle_fs_read_file_proxy_deprecated(
	// Parameters from Cocoon's fs.readFile call
	_params:Value,
) -> Result<Value, String> {
	log::warn!(
		"[FS Handler Proxy - DEPRECATED] handle_fs_read_file_proxy_deprecated called. This handler is non-functional."
	);

	Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.to_string(), None))
}

/// **DEPRECATED:** Use `vscode.workspace.fs.writeFile` (via
/// `handlers::workspace_fs_api::handle_write_file`) instead.
///
/// This function is a stub and will always return an error.
pub async fn handle_fs_write_file_proxy_deprecated(
	// Parameters from Cocoon's fs.writeFile call
	_params:Value,
) -> Result<Value, String> {
	log::warn!(
		"[FS Handler Proxy - DEPRECATED] handle_fs_write_file_proxy_deprecated called. This handler is non-functional."
	);

	Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.to_string(), None))
}

/// **DEPRECATED:** Use `vscode.workspace.fs.createDirectory` (via
/// `handlers::workspace_fs_api::handle_create_directory`) instead.
///
/// This function is a stub and will always return an error.
pub async fn handle_fs_mkdir_proxy_deprecated(
	// Parameters from Cocoon's fs.mkdir call
	_params:Value,
) -> Result<Value, String> {
	log::warn!(
		"[FS Handler Proxy - DEPRECATED] handle_fs_mkdir_proxy_deprecated called. This handler is non-functional."
	);

	Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.to_string(), None))
}

/// **DEPRECATED:** Use `vscode.workspace.fs.delete` (via
/// `handlers::workspace_fs_api::handle_delete`) instead. Note that `unlink`
/// typically implies deleting a file, while `vscode.workspace.fs.delete` can
/// handle files and directories (with `recursive` option).
///
/// This function is a stub and will always return an error.
pub async fn handle_fs_unlink_proxy_deprecated(
	// Parameters from Cocoon's fs.unlink call
	_params:Value,
) -> Result<Value, String> {
	log::warn!(
		"[FS Handler Proxy - DEPRECATED] handle_fs_unlink_proxy_deprecated called. This handler is non-functional."
	);

	Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.to_string(), None))
}

// ... Add similar deprecated stubs for any other fs_* proxy handlers that might
// have existed, for example:
//
// /// **DEPRECATED:** Use `vscode.workspace.fs.readDirectory`
// pub async fn handle_fs_readdir_proxy_deprecated(_params: Value) ->
// Result<Value, String> {     log::warn!("[FS Handler Proxy - DEPRECATED]
// handle_fs_readdir_proxy_deprecated called.");

//     Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.
// to_string(), None)) }

// /// **DEPRECATED:** Use `vscode.workspace.fs.rename`
// pub async fn handle_fs_rename_proxy_deprecated(_params: Value) ->
// Result<Value, String> {     log::warn!("[FS Handler Proxy - DEPRECATED]
// handle_fs_rename_proxy_deprecated called.");

//     Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.
// to_string(), None)) }

// /// **DEPRECATED:** Use `vscode.workspace.fs.copy`
// pub async fn handle_fs_copy_proxy_deprecated(_params: Value) -> Result<Value,

// String> {     log::warn!("[FS Handler Proxy - DEPRECATED]
// handle_fs_copy_proxy_deprecated called.");

//     Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.
// to_string(), None)) }

// /// **DEPRECATED:** Use `vscode.workspace.fs.stat` (fs.exists is often
// discouraged in Node in favor of try-catch with actual operation) pub async fn
// handle_fs_exists_proxy_deprecated(_params: Value) -> Result<Value, String> {

//     log::warn!("[FS Handler Proxy - DEPRECATED]
// handle_fs_exists_proxy_deprecated called.");

//     Err(create_deprecated_error_string(DEPRECATED_NATIVE_FS_ERROR_MSG.
// to_string(), None)) }

// ----- END: DEPRECATED Element/Mountain/src/Handler/native_fs.rs -----
