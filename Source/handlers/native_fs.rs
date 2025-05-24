// ---------------------------------------------------------------------------------------------
// Mountain Native FS Handlers (handlers/native_fs.rs) - DEPRECATED (Path A)
// --------------------------------------------------------------------------------------------
// This file previously handled low-level filesystem requests proxied directly
// from Cocoon's Node 'fs' module shim (using `fs_*` method names).
//
// CURRENT STATUS (Path A with workspace.fs): **DEPRECATED/UNUSED**
// With the implementation of the `vscode.workspace.fs` API shim
// (`fs-api-shim.js`) and corresponding Mountain handlers
// (`handlers/workspace_fs_api.rs`) which delegate to the Environment
// implementation (`environment.rs`), direct proxying of the Node 'fs'
// module implemented here is **not used** in the primary MVP Path A flow.
//
// Extensions *should* use `vscode.workspace.fs`. If direct Node 'fs' access
// were re-enabled, these handlers would need significant updates, including
// security checks and alignment with the Environment's FS capabilities.
//
// This file is kept for historical reference only and its functions should not
// be called.
// --------------------------------------------------------------------------------------------

// ----- START: DEPRECATED Element/Mountain/src/handlers/native_fs.rs -----
// NOTE: This entire file is DEPRECATED for Path A (vscode.workspace.fs)

use std::path::PathBuf;

// These imports might not be needed if the file is truly unused, but kept for context
use Land_River::api as river_api;
use Land_Sun::api as sun_api;
// Use log crate if logging warnings about deprecation
use log;
use serde_json::{Value, json};
use tauri::{AppHandle, Runtime, Window};

const DEPRECATED_ERROR_MSG:&str = "Native FS handlers are deprecated; use workspace.fs API via effects/environment.";

// --- Helper Functions (remain unchanged, but unused in Path A) ---
fn create_error_string(message:String, code:Option<&str>) -> String {
	json!({ "message": message, "code": code.unwrap_or("EUNKNOWN") }).to_string()
}

// Returns error immediately as path parsing is not relevant for deprecated
// handlers
fn path_from_uri_components(_params:&Value) -> Result<PathBuf, String> {
	Err(create_error_string(DEPRECATED_ERROR_MSG.to_string(), Some("ENOSYS")))
}

// --- Direct Action Handlers (remain deprecated stubs) ---
/// **DEPRECATED:** Use `workspace.fs` API effects instead.
pub async fn handle_read_file<R:Runtime>(/* ... */) -> Result<Value, String> {
	log::warn!("[FS Handler - DEPRECATED] Direct handle_read_file called.");

	Err(create_error_string(DEPRECATED_ERROR_MSG.to_string(), Some("ENOSYS")))
}

/// **DEPRECATED:** Use `workspace.fs` API effects instead.
pub async fn handle_write_file<R:Runtime>(/* ... */) -> Result<Value, String> {
	log::warn!("[FS Handler - DEPRECATED] Direct handle_write_file called.");

	Err(create_error_string(DEPRECATED_ERROR_MSG.to_string(), Some("ENOSYS")))
}

// --- Proxied Handlers (remain deprecated stubs) ---
/// **DEPRECATED:** Use `workspace.fs` API effects instead.
pub async fn handle_fs_stat(_params:Value) -> Result<Value, String> {
	log::warn!("[FS Handler Proxy - DEPRECATED] handle_fs_stat called.");

	Err(create_error_string(DEPRECATED_ERROR_MSG.to_string(), Some("ENOSYS")))
}

/// **DEPRECATED:** Use `workspace.fs` API effects instead.
pub async fn handle_fs_realpath(_params:Value) -> Result<Value, String> {
	log::warn!("[FS Handler Proxy - DEPRECATED] handle_fs_realpath called.");

	Err(create_error_string(DEPRECATED_ERROR_MSG.to_string(), Some("ENOSYS")))
}

/// **DEPRECATED:** Use `workspace.fs` API effects instead.
pub async fn handle_fs_read_file_proxy(_params:Value) -> Result<Value, String> {
	log::warn!("[FS Handler Proxy - DEPRECATED] handle_fs_read_file_proxy called.");

	Err(create_error_string(DEPRECATED_ERROR_MSG.to_string(), Some("ENOSYS")))
}

/// **DEPRECATED:** Use `workspace.fs` API effects instead.
pub async fn handle_fs_write_file_proxy(_params:Value) -> Result<Value, String> {
	log::warn!("[FS Handler Proxy - DEPRECATED] handle_fs_write_file_proxy called.");

	Err(create_error_string(DEPRECATED_ERROR_MSG.to_string(), Some("ENOSYS")))
}

/// **DEPRECATED:** Use `workspace.fs` API effects instead.
pub async fn handle_fs_mkdir_proxy(_params:Value) -> Result<Value, String> {
	log::warn!("[FS Handler Proxy - DEPRECATED] handle_fs_mkdir_proxy called.");

	Err(create_error_string(DEPRECATED_ERROR_MSG.to_string(), Some("ENOSYS")))
}

/// **DEPRECATED:** Use `workspace.fs` API effects instead.
pub async fn handle_fs_unlink_proxy(_params:Value) -> Result<Value, String> {
	log::warn!("[FS Handler Proxy - DEPRECATED] handle_fs_unlink_proxy called.");

	Err(create_error_string(DEPRECATED_ERROR_MSG.to_string(), Some("ENOSYS")))
}

// ... Add similar stubs for any other fs_* proxy handlers that existed ...
// pub async fn handle_fs_readdir_proxy(...) -> Result<Value, String> { Err(...)
// } pub async fn handle_fs_rename_proxy(...) -> Result<Value, String> {

// Err(...) } pub async fn handle_fs_copy_proxy(...) -> Result<Value, String> {

// Err(...) } pub async fn handle_fs_exists_proxy(...) -> Result<Value, String>
// { Err(...) }

// ----- END: DEPRECATED Element/Mountain/src/handlers/native_fs.rs -----
