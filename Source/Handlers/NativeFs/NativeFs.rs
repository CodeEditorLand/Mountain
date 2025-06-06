
// Defines deprecated native filesystem handlers. These functions are
// placeholders and should not be used. All filesystem operations must go
// through the `vscode.workspace.fs` API, which is implemented by the `FsReader`
// and `FsWriter` traits in the application environment.

#![allow(non_snake_case, non_camel_case_types)]

use std::path::PathBuf;

use log;
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Handlers::ErrorUtils; // Assuming ErrorUtils will be PascalCased

const DEPRECATED_NATIVE_FS_ERROR_MESSAGE:&str =
	"Native FS direct proxy handlers are deprecated; use the vscode.workspace.fs API via effects/environment.";

/// Creates a standardized error string for deprecated functions.
fn CreateDeprecatedErrorString(Message:String, Code:Option<&str>) -> String {
	ErrorUtils::RpcErrorString(Message, Code.or(Some("ENOSYS_DEPRECATED")))
}

// All functions below are deprecated and return an error indicating they are
// non-functional.

pub async fn HandleReadFileDeprecated<R:Runtime>() -> Result<Value, String> {
	log::warn!("[FsHandler Deprecated] HandleReadFileDeprecated called. This handler is non-functional.");
	Err(CreateDeprecatedErrorString(
		DEPRECATED_NATIVE_FS_ERROR_MESSAGE.to_string(),
		None,
	))
}

pub async fn HandleWriteFileDeprecated<R:Runtime>() -> Result<Value, String> {
	log::warn!("[FsHandler Deprecated] HandleWriteFileDeprecated called. This handler is non-functional.");
	Err(CreateDeprecatedErrorString(
		DEPRECATED_NATIVE_FS_ERROR_MESSAGE.to_string(),
		None,
	))
}

pub async fn HandleFsStatDeprecated(_Parameters:Value) -> Result<Value, String> {
	log::warn!("[FsHandler Deprecated] HandleFsStatDeprecated called. This handler is non-functional.");
	Err(CreateDeprecatedErrorString(
		DEPRECATED_NATIVE_FS_ERROR_MESSAGE.to_string(),
		None,
	))
}

pub async fn HandleFsRealpathDeprecated(_Parameters:Value) -> Result<Value, String> {
	log::warn!("[FsHandler Deprecated] HandleFsRealpathDeprecated called. This handler is non-functional.");
	Err(CreateDeprecatedErrorString(
		DEPRECATED_NATIVE_FS_ERROR_MESSAGE.to_string(),
		None,
	))
}

pub async fn HandleFsReadFileProxyDeprecated(_Parameters:Value) -> Result<Value, String> {
	log::warn!("[FsHandler Deprecated] HandleFsReadFileProxyDeprecated called. This handler is non-functional.");
	Err(CreateDeprecatedErrorString(
		DEPRECATED_NATIVE_FS_ERROR_MESSAGE.to_string(),
		None,
	))
}

pub async fn HandleFsWriteFileProxyDeprecated(_Parameters:Value) -> Result<Value, String> {
	log::warn!("[FsHandler Deprecated] HandleFsWriteFileProxyDeprecated called. This handler is non-functional.");
	Err(CreateDeprecatedErrorString(
		DEPRECATED_NATIVE_FS_ERROR_MESSAGE.to_string(),
		None,
	))
}

pub async fn HandleFsMkdirProxyDeprecated(_Parameters:Value) -> Result<Value, String> {
	log::warn!("[FsHandler Deprecated] HandleFsMkdirProxyDeprecated called. This handler is non-functional.");
	Err(CreateDeprecatedErrorString(
		DEPRECATED_NATIVE_FS_ERROR_MESSAGE.to_string(),
		None,
	))
}

pub async fn HandleFsUnlinkProxyDeprecated(_Parameters:Value) -> Result<Value, String> {
	log::warn!("[FsHandler Deprecated] HandleFsUnlinkProxyDeprecated called. This handler is non-functional.");
	Err(CreateDeprecatedErrorString(
		DEPRECATED_NATIVE_FS_ERROR_MESSAGE.to_string(),
		None,
	))
}
