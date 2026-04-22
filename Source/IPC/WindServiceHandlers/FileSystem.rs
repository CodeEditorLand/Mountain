#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! File system handlers — both legacy (runtime-routed) and native (URI-aware).

use std::{path::PathBuf, sync::Arc};

use serde_json::{Value, json};

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};

use crate::{dev_log, RunTime::ApplicationRunTime::ApplicationRunTime};

use super::Utilities::{extract_path_from_arg, metadata_to_istat};

// ============================================================================
// Legacy handlers (via FileSystemReader / FileSystemWriter traits)
// ============================================================================

/// Handler for file read requests (legacy)
pub async fn handle_file_read(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let content = provider
		.ReadFile(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to read file: {}", e))?;

	dev_log!("vfs", "read: {} ({} bytes)", path, content.len());
	Ok(json!(content))
}

/// Handler for file write requests (legacy)
pub async fn handle_file_write(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let content = args
		.get(1)
		.ok_or("Missing file content".to_string())?
		.as_str()
		.ok_or("File content must be a string".to_string())?;

	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.WriteFile(&PathBuf::from(path), content.as_bytes().to_vec(), true, true)
		.await
		.map_err(|e:CommonError| format!("Failed to write file: {}", e))?;

	dev_log!("vfs", "written: {} ({} bytes)", path, content.len());
	Ok(Value::Null)
}

/// Handler for file stat requests (legacy)
pub async fn handle_file_stat(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let stats = provider
		.StatFile(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to stat file: {}", e))?;

	dev_log!("vfs", "legacy_stat: {}", path);
	Ok(json!(stats))
}

/// Handler for file exists requests (legacy)
pub async fn handle_file_exists(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let exists = provider.StatFile(&PathBuf::from(path)).await.is_ok();

	dev_log!("vfs", "exists: {} = {}", path, exists);
	Ok(json!(exists))
}

/// Handler for file delete requests (legacy)
pub async fn handle_file_delete(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.Delete(&PathBuf::from(path), false, false)
		.await
		.map_err(|e:CommonError| format!("Failed to delete file: {}", e))?;

	dev_log!("vfs", "deleted: {}", path);
	Ok(Value::Null)
}

/// Handler for file copy requests (legacy)
pub async fn handle_file_copy(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let source = args
		.get(0)
		.ok_or("Missing source path".to_string())?
		.as_str()
		.ok_or("Source path must be a string".to_string())?;

	let destination = args
		.get(1)
		.ok_or("Missing destination path".to_string())?
		.as_str()
		.ok_or("Destination path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.Copy(&PathBuf::from(source), &PathBuf::from(destination), false)
		.await
		.map_err(|e:CommonError| format!("Failed to copy file: {} -> {}", source, destination))?;

	dev_log!("vfs", "copied: {} -> {}", source, destination);
	Ok(Value::Null)
}

/// Handler for file move requests (legacy)
pub async fn handle_file_move(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let source = args
		.get(0)
		.ok_or("Missing source path".to_string())?
		.as_str()
		.ok_or("Source path must be a string".to_string())?;

	let destination = args
		.get(1)
		.ok_or("Missing destination path".to_string())?
		.as_str()
		.ok_or("Destination path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.Rename(&PathBuf::from(source), &PathBuf::from(destination), false)
		.await
		.map_err(|e:CommonError| format!("Failed to move file: {} -> {}", source, destination))?;

	dev_log!("vfs", "moved: {} -> {}", source, destination);
	Ok(Value::Null)
}

/// Handler for directory creation requests (legacy)
pub async fn handle_file_mkdir(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing directory path".to_string())?
		.as_str()
		.ok_or("Directory path must be a string".to_string())?;

	let recursive = args.get(1).and_then(|v| v.as_bool()).unwrap_or(true);

	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.CreateDirectory(&PathBuf::from(path), recursive)
		.await
		.map_err(|e:CommonError| format!("Failed to create directory: {}", e))?;

	dev_log!("vfs", "mkdir: {} (recursive: {})", path, recursive);
	Ok(Value::Null)
}

/// Handler for directory reading requests (legacy)
pub async fn handle_file_readdir(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing directory path".to_string())?
		.as_str()
		.ok_or("Directory path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let entries = provider
		.ReadDirectory(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to read directory: {}", e))?;

	dev_log!("vfs", "readdir_legacy: {} ({} entries)", path, entries.len());
	Ok(json!(entries))
}

/// Handler for binary file read requests (legacy)
pub async fn handle_file_read_binary(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let content = provider
		.ReadFile(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to read binary file: {}", e))?;

	dev_log!("vfs", "readBinary: {} ({} bytes)", path, content.len());
	Ok(json!(content))
}

/// Handler for binary file write requests (legacy)
pub async fn handle_file_write_binary(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let content = args
		.get(1)
		.ok_or("Missing file content".to_string())?
		.as_str()
		.ok_or("File content must be a string".to_string())?;

	let content_bytes = content.as_bytes().to_vec();
	let content_len = content_bytes.len();

	let provider:Arc<dyn FileSystemWriter> = runtime.Environment.Require();

	provider
		.WriteFile(&PathBuf::from(path), content_bytes.clone(), true, true)
		.await
		.map_err(|e:CommonError| format!("Failed to write binary file: {}", e))?;

	dev_log!("vfs", "writeBinary: {} ({} bytes)", path, content_len);
	Ok(Value::Null)
}

// ============================================================================
// Native handlers (URI-aware, direct tokio::fs)
// ============================================================================

/// Read file with URI arg support.
/// Returns { buffer: number[] } where buffer is the raw byte content.
/// VS Code's DiskFileSystemProviderClient wraps this with VSBuffer.wrap().
pub async fn handle_file_read_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

	dev_log!("vfs", "readFile: {}", Path);

	let Bytes = tokio::fs::read(&Path)
		.await
		.map_err(|E| format!("Failed to read file: {} (path: {})", E, Path))?;

	dev_log!("vfs", "readFile OK: {} ({} bytes)", Path, Bytes.len());

	// Return as { buffer: [byte, byte, ...] } - VS Code reconstructs as VSBuffer
	let ByteArray:Vec<Value> = Bytes.iter().map(|B| json!(*B)).collect();
	Ok(json!({ "buffer": ByteArray }))
}

/// Write file with URI arg support
pub async fn handle_file_write_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

	// args[1] is VSBuffer (content), args[2] is options
	let Content = args.get(1).ok_or("Missing file content")?;

	let Bytes = if let Some(S) = Content.as_str() {
		S.as_bytes().to_vec()
	} else if let Some(Obj) = Content.as_object() {
		// VSBuffer wraps { buffer: Uint8Array } - extract bytes
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

	// Ensure parent directory exists
	if let Some(Parent) = std::path::Path::new(&Path).parent() {
		tokio::fs::create_dir_all(Parent).await.ok();
	}

	tokio::fs::write(&Path, &Bytes)
		.await
		.map_err(|E| format!("Failed to write file: {} (path: {})", E, Path))?;

	Ok(Value::Null)
}

/// Rename/move file with URI arg support
pub async fn handle_file_rename_native(args:Vec<Value>) -> Result<Value, String> {
	let Source = extract_path_from_arg(args.get(0).ok_or("Missing source path")?)?;
	let Target = extract_path_from_arg(args.get(1).ok_or("Missing target path")?)?;

	tokio::fs::rename(&Source, &Target)
		.await
		.map_err(|E| format!("Failed to rename: {} -> {} ({})", Source, Target, E))?;

	Ok(Value::Null)
}

/// Resolve real path (follow symlinks)
pub async fn handle_file_realpath(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing path")?)?;

	let Canonical = tokio::fs::canonicalize(&Path)
		.await
		.map_err(|E| format!("Failed to realpath: {} ({})", Path, E))?;

	Ok(json!({
		"scheme": "file",
		"path": Canonical.to_string_lossy(),
		"authority": ""
	}))
}

/// Clone file (copy with metadata)
pub async fn handle_file_clone_native(args:Vec<Value>) -> Result<Value, String> {
	let Source = extract_path_from_arg(args.get(0).ok_or("Missing source path")?)?;
	let Target = extract_path_from_arg(args.get(1).ok_or("Missing target path")?)?;

	tokio::fs::copy(&Source, &Target)
		.await
		.map_err(|E| format!("Failed to clone: {} -> {} ({})", Source, Target, E))?;

	Ok(Value::Null)
}

/// Stat file - pure stat, no side effects. Returns IStat shape.
pub async fn handle_file_stat_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

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
pub async fn handle_file_exists_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

	Ok(json!(tokio::fs::try_exists(&Path).await.unwrap_or(false)))
}

/// Delete file or directory with URI arg support
pub async fn handle_file_delete_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing file path")?)?;

	// Options may include { recursive, useTrash }
	let Recursive = args
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
pub async fn handle_file_mkdir_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing directory path")?)?;

	tokio::fs::create_dir_all(&Path)
		.await
		.map_err(|E| format!("Failed to mkdir: {} ({})", Path, E))?;

	Ok(Value::Null)
}

/// Read directory contents with URI arg support.
/// Returns array of [name, fileType] tuples matching VS Code's ReadDirResult.
pub async fn handle_file_readdir_native(args:Vec<Value>) -> Result<Value, String> {
	let Path = extract_path_from_arg(args.get(0).ok_or("Missing directory path")?)?;

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
