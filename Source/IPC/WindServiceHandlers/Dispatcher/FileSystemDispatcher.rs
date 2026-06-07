//! FileSystem command dispatcher - handles native and managed file operations.

use serde_json::Value;

use crate::FileSystem::{
	Managed::{
		FileCopy::Fn as FileCopy,
		FileDelete::Fn as FileDelete,
		FileExists::Fn as FileExists,
		FileMkdir::Fn as FileMkdir,
		FileMove::Fn as FileMove,
		FileRead::Fn as FileRead,
		FileReadBinary::Fn as FileReadBinary,
		FileReaddir::Fn as FileReaddir,
		FileStat::Fn as FileStat,
		FileWrite::Fn as FileWrite,
		FileWriteBinary::Fn as FileWriteBinary,
	},
	Native::{
		FileCloneNative::Fn as FileCloneNative,
		FileCloseFd::Fn as FileCloseFd,
		FileDeleteNative::Fn as FileDeleteNative,
		FileExistsNative::Fn as FileExistsNative,
		FileMkdirNative::Fn as FileMkdirNative,
		FileOpenFd::Fn as FileOpenFd,
		FileReadNative::Fn as FileReadNative,
		FileReaddirNative::Fn as FileReaddirNative,
		FileRealpath::Fn as FileRealpath,
		FileRenameNative::Fn as FileRenameNative,
		FileStatNative::Fn as FileStatNative,
		FileUnwatch::Fn as FileUnwatch,
		FileWatch::Fn as FileWatch,
		FileWriteNative::Fn as FileWriteNative,
	},
};

/// Dispatches file system commands.
///
/// Handled commands:
/// - `file:read` / `file:readFile` -> FileReadNative
/// - `file:write` / `file:writeFile` -> FileWriteNative
/// - `file:stat` -> FileStatNative
/// - `file:exists` -> FileExistsNative
/// - `file:delete` -> FileDeleteNative
/// - `file:copy` -> FileCloneNative
/// - `file:move` / `file:rename` -> FileRenameNative
/// - `file:mkdir` -> FileMkdirNative
/// - `file:readdir` -> FileReaddirNative
/// - `file:readBinary` -> FileReadBinary
/// - `file:writeBinary` -> FileWriteBinary
/// - `file:watch` -> FileWatch
/// - `file:unwatch` -> FileUnwatch
/// - `file:realpath` -> FileRealpath
/// - `file:open` -> FileOpenFd
/// - `file:close` -> FileCloseFd
/// - `file:cloneFile` -> FileCloneNative
pub async fn dispatch_filesystem(
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"file:read" | "file:readFile" => FileReadNative(arguments).await,

		"file:write" | "file:writeFile" => FileWriteNative(arguments).await,

		"file:stat" => FileStatNative(arguments).await,

		"file:exists" => FileExistsNative(arguments).await,

		"file:delete" => FileDeleteNative(arguments).await,

		"file:copy" => FileCloneNative(arguments).await,

		"file:move" | "file:rename" => FileRenameNative(arguments).await,

		"file:mkdir" => FileMkdirNative(arguments).await,

		"file:readdir" => FileReaddirNative(arguments).await,

		"file:readBinary" => FileReadBinary(runtime.clone(), arguments).await,

		"file:writeBinary" => FileWriteBinary(runtime.clone(), arguments).await,

		"file:watch" => FileWatch(runtime.clone(), arguments).await,

		"file:unwatch" => FileUnwatch(runtime.clone(), arguments).await,

		"file:realpath" => FileRealpath(arguments).await,

		"file:open" => FileOpenFd(arguments).await,

		"file:close" => FileCloseFd(arguments).await,

		"file:cloneFile" => FileCloneNative(arguments).await,

		_ => Err(format!("Unknown filesystem command: {}", command)),
	}
}
