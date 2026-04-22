#![allow(non_snake_case)]

//! File System domain handlers for Wind IPC.
//!
//! Contains both legacy runtime-based handlers and native URI-aware handlers
//! used by VS Code's DiskFileSystemProviderClient.

use std::{path::PathBuf, sync::Arc};

use serde_json::{Value, json};
use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};

use crate::{
	IPC::UriComponents::FromFilePath as UriFromFilePath,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};
use super::{extract_path_from_arg, metadata_to_istat};

// ============================================================================
// Legacy runtime-based handlers
// ============================================================================

/// Handler for file read requests
pub async fn handle_file_read(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Path = Args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let Provider:Arc<dyn FileSystemReader> = Runtime.Environment.Require();

	let Content = Provider
		.ReadFile(&PathBuf::from(Path))
		.await
		.map_err(|E| format!("Failed to read file: {}", E))?;

	dev_log!("vfs", "read: {} ({} bytes)", Path, Content.len());
	Ok(json!(Content))
}

/// Handler for file write requests
pub async fn handle_file_write(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Path = Args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let Content = Args
		.get(1)
		.ok_or("Missing file content".to_string())?
		.as_str()
		.ok_or("File content must be a string".to_string())?;

	let Provider:Arc<dyn FileSystemWriter> = Runtime.Environment.Require();

	Provider
		.WriteFile(&PathBuf::from(Path), Content.as_bytes().to_vec(), true, true)
		.await
		.map_err(|E:CommonError| format!("Failed to write file: {}", E))?;

	dev_log!("vfs", "written: {} ({} bytes)", Path, Content.len());
	Ok(Value::Null)
}

/// Handler for file stat requests
pub async fn handle_file_stat(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Path = Args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let Provider:Arc<dyn FileSystemReader> = Runtime.Environment.Require();

	let Stats = Provider
		.StatFile(&PathBuf::from(Path))
		.await
		.map_err(|E| format!("Failed to stat file: {}", E))?;

	dev_log!("vfs", "legacy_stat: {}", Path);
	Ok(json!(Stats))
}

/// Handler for file exists requests
pub async fn handle_file_exists(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Path = Args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let Provider:Arc<dyn FileSystemReader> = Runtime.Environment.Require();

	let Exists = Provider.StatFile(&PathBuf::from(Path)).await.is_ok();

	dev_log!("vfs", "exists: {} = {}", Path, Exists);
	Ok(json!(Exists))
}

/// Handler for file delete requests
pub async fn handle_file_delete(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Path = Args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let Provider:Arc<dyn FileSystemWriter> = Runtime.Environment.Require();

	Provider
		.Delete(&PathBuf::from(Path), false, false)
		.await
		.map_err(|E:CommonError| format!("Failed to delete file: {}", E))?;

	dev_log!("vfs", "deleted: {}", Path);
	Ok(Value::Null)
}

/// Handler for file copy requests
pub async fn handle_file_copy(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Source = Args
		.get(0)
		.ok_or("Missing source path".to_string())?
		.as_str()
		.ok_or("Source path must be a string".to_string())?;

	let Destination = Args
		.get(1)
		.ok_or("Missing destination path".to_string())?
		.as_str()
		.ok_or("Destination path must be a string".to_string())?;

	let Provider:Arc<dyn FileSystemWriter> = Runtime.Environment.Require();

	Provider
		.Copy(&PathBuf::from(Source), &PathBuf::from(Destination), false)
		.await
		.map_err(|_E:CommonError| format!("Failed to copy file: {} -> {}", Source, Destination))?;

	dev_log!("vfs", "copied: {} -> {}", Source, Destination);
	Ok(Value::Null)
}

/// Handler for file move requests
pub async fn handle_file_move(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Source = Args
		.get(0)
		.ok_or("Missing source path".to_string())?
		.as_str()
		.ok_or("Source path must be a string".to_string())?;

	let Destination = Args
		.get(1)
		.ok_or("Missing destination path".to_string())?
		.as_str()
		.ok_or("Destination path must be a string".to_string())?;

	let Provider:Arc<dyn FileSystemWriter> = Runtime.Environment.Require();

	Provider
		.Rename(&PathBuf::from(Source), &PathBuf::from(Destination), false)
		.await
		.map_err(|_E:CommonError| format!("Failed to move file: {} -> {}", Source, Destination))?;

	dev_log!("vfs", "moved: {} -> {}", Source, Destination);
	Ok(Value::Null)
}

/// Handler for directory creation requests
pub async fn handle_file_mkdir(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Path = Args
		.get(0)
		.ok_or("Missing directory path".to_string())?
		.as_str()
		.ok_or("Directory path must be a string".to_string())?;

	let Recursive = Args.get(1).and_then(|V| V.as_bool()).unwrap_or(true);

	let Provider:Arc<dyn FileSystemWriter> = Runtime.Environment.Require();

	Provider
		.CreateDirectory(&PathBuf::from(Path), Recursive)
		.await
		.map_err(|E:CommonError| format!("Failed to create directory: {}", E))?;

	dev_log!("vfs", "mkdir: {} (recursive: {})", Path, Recursive);
	Ok(Value::Null)
}

/// Handler for directory reading requests
pub async fn handle_file_readdir(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Path = Args
		.get(0)
		.ok_or("Missing directory path".to_string())?
		.as_str()
		.ok_or("Directory path must be a string".to_string())?;

	let Provider:Arc<dyn FileSystemReader> = Runtime.Environment.Require();

	let Entries = Provider
		.ReadDirectory(&PathBuf::from(Path))
		.await
		.map_err(|E| format!("Failed to read directory: {}", E))?;

	dev_log!("vfs", "readdir_legacy: {} ({} entries)", Path, Entries.len());
	Ok(json!(Entries))
}

/// Handler for binary file read requests
pub async fn handle_file_read_binary(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Path = Args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let Provider:Arc<dyn FileSystemReader> = Runtime.Environment.Require();

	let Content = Provider
		.ReadFile(&PathBuf::from(Path))
		.await
		.map_err(|E| format!("Failed to read binary file: {}", E))?;

	dev_log!("vfs", "readBinary: {} ({} bytes)", Path, Content.len());
	Ok(json!(Content))
}

/// Handler for binary file write requests
pub async fn handle_file_write_binary(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Path = Args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let Content = Args
		.get(1)
		.ok_or("Missing file content".to_string())?
		.as_str()
		.ok_or("File content must be a string".to_string())?;

	let ContentBytes = Content.as_bytes().to_vec();
	let ContentLength = ContentBytes.len();

	let Provider:Arc<dyn FileSystemWriter> = Runtime.Environment.Require();

	Provider
		.WriteFile(&PathBuf::from(Path), ContentBytes.clone(), true, true)
		.await
		.map_err(|E:CommonError| format!("Failed to write binary file: {}", E))?;

	dev_log!("vfs", "writeBinary: {} ({} bytes)", Path, ContentLength);
	Ok(Value::Null)
}

// ============================================================================
// Native URI-aware handlers (used by VS Code DiskFileSystemProviderClient)
// ============================================================================

/// Read file with URI arg support (VS Code sends { scheme, path } objects)
/// Returns { buffer: number[] } where buffer is the raw byte content.
pub async fn handle_file_read_native(Args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Args.get(0).ok_or("Missing file path")?)?;

	dev_log!("vfs", "readFile: {}", Path);

	let Bytes = tokio::fs::read(&Path)
		.await
		.map_err(|E| format!("Failed to read file: {} (path: {})", E, Path))?;

	dev_log!("vfs", "readFile OK: {} ({} bytes)", Path, Bytes.len());

	let ByteArray:Vec<Value> = Bytes.iter().map(|B| json!(*B)).collect();
	Ok(json!({ "buffer": ByteArray }))
}

/// Write file with URI arg support
pub async fn handle_file_write_native(Args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Args.get(0).ok_or("Missing file path")?)?;

	let Content = Args.get(1).ok_or("Missing file content")?;

	let Bytes = if let Some(S) = Content.as_str() {
		S.as_bytes().to_vec()
	} else if let Some(Obj) = Content.as_object() {
		if let Some(Buf) = Obj.get("buffer") {
			if let Some(Arr) = Buf.as_array() {
				Arr.iter().filter_map(|V| V.as_u64().map(|N| N as u8)).collect()
			} else if let Some(S) = Buf.as_str() {
				S.as_bytes().to_vec()
			} else {
				return Err("Unsupported buffer format".to_string());
			}
		} else {
			serde_json::to_string(Content).unwrap_or_default().into_bytes()
		}
	} else {
		return Err("File content must be a string or VSBuffer".to_string());
	};

	if let Some(Parent) = std::path::Path::new(&Path).parent() {
		tokio::fs::create_dir_all(Parent).await.ok();
	}

	tokio::fs::write(&Path, &Bytes)
		.await
		.map_err(|E| format!("Failed to write file: {} (path: {})", E, Path))?;

	Ok(Value::Null)
}

/// Stat file - pure stat, no side effects. Returns IStat shape.
pub async fn handle_file_stat_native(Args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Args.get(0).ok_or("Missing file path")?)?;

	// Skip the per-stat `stat: <path>` echo for known-optional probes -
	// those paths are handled by `DebugOnce` on the ENOENT branch instead.
	if !crate::IPC::DevLog::IsBenignEnoent(&Path) {
		dev_log!("vfs", "stat: {}", Path);
	}

	let Metadata = tokio::fs::symlink_metadata(&Path).await.map_err(|E| {
		if crate::IPC::DevLog::IsBenignEnoent(&Path) {
			crate::IPC::DevLog::DebugOnce(
				"vfs",
				&format!("stat-enoent:{}", Path),
				&format!("stat ENOENT (benign): {}", Path),
			);
		} else {
			dev_log!("vfs", "stat ENOENT: {}", Path);
		}
		format!("Failed to stat file: {} (path: {})", E, Path)
	})?;

	if !crate::IPC::DevLog::IsBenignEnoent(&Path) {
		dev_log!("vfs", "stat OK: {} (dir={})", Path, Metadata.is_dir());
	}
	Ok(metadata_to_istat(&Metadata))
}

/// Check file existence with URI arg support
pub async fn handle_file_exists_native(Args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Args.get(0).ok_or("Missing file path")?)?;

	Ok(json!(tokio::fs::try_exists(&Path).await.unwrap_or(false)))
}

/// Delete file or directory with URI arg support
pub async fn handle_file_delete_native(Args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Args.get(0).ok_or("Missing file path")?)?;

	let Recursive = Args
		.get(1)
		.and_then(|V| V.as_object())
		.and_then(|O| O.get("recursive"))
		.and_then(|V| V.as_bool())
		.unwrap_or(false);

	let PathBuf = std::path::Path::new(&Path);

	if PathBuf.is_dir() {
		if Recursive {
			tokio::fs::remove_dir_all(&Path).await
		} else {
			tokio::fs::remove_dir(&Path).await
		}
	} else {
		tokio::fs::remove_file(&Path).await
	}
	.map_err(|E| format!("Failed to delete: {} ({})", Path, E))?;

	Ok(Value::Null)
}

/// Create directory with URI arg support
pub async fn handle_file_mkdir_native(Args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Args.get(0).ok_or("Missing directory path")?)?;

	tokio::fs::create_dir_all(&Path)
		.await
		.map_err(|E| format!("Failed to mkdir: {} ({})", Path, E))?;

	Ok(Value::Null)
}

/// Read directory contents with URI arg support
/// Returns array of [name, fileType] tuples matching VS Code's ReadDirResult
pub async fn handle_file_readdir_native(Args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Args.get(0).ok_or("Missing directory path")?)?;

	dev_log!("vfs", "readdir: {}", Path);

	let mut Entries = tokio::fs::read_dir(&Path)
		.await
		.map_err(|E| format!("Failed to readdir: {} ({})", Path, E))?;

	let mut Result = Vec::new();

	while let Some(Entry) = Entries.next_entry().await.map_err(|E| E.to_string())? {
		let Name = Entry.file_name().to_string_lossy().to_string();
		let FileType = Entry.file_type().await.map_err(|E| E.to_string())?;

		let TypeValue = if FileType.is_symlink() {
			64 // SymbolicLink
		} else if FileType.is_dir() {
			2 // Directory
		} else {
			1 // File
		};

		Result.push(json!([Name, TypeValue]));
	}

	Ok(json!(Result))
}

/// Rename/move file with URI arg support
pub async fn handle_file_rename_native(Args:Vec<Value>) -> Result<Value, String> {
	let Source = extract_path_from_arg(Args.get(0).ok_or("Missing source path")?)?;
	let Target = extract_path_from_arg(Args.get(1).ok_or("Missing target path")?)?;

	tokio::fs::rename(&Source, &Target)
		.await
		.map_err(|E| format!("Failed to rename: {} -> {} ({})", Source, Target, E))?;

	Ok(Value::Null)
}

/// Resolve real path (follow symlinks)
pub async fn handle_file_realpath(Args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(Args.get(0).ok_or("Missing path")?)?;

	let Canonical = tokio::fs::canonicalize(&Path)
		.await
		.map_err(|E| format!("Failed to realpath: {} ({})", Path, E))?;

	// VS Code-marked UriComponents (`$mid: 1`) for the renderer reviver.
	Ok(UriFromFilePath(Canonical.to_string_lossy()))
}

/// Clone file (copy with metadata)
pub async fn handle_file_clone_native(Args:Vec<Value>) -> Result<Value, String> {
	let Source = extract_path_from_arg(Args.get(0).ok_or("Missing source path")?)?;
	let Target = extract_path_from_arg(Args.get(1).ok_or("Missing target path")?)?;

	tokio::fs::copy(&Source, &Target)
		.await
		.map_err(|E| format!("Failed to clone: {} -> {} ({})", Source, Target, E))?;

	Ok(Value::Null)
}
