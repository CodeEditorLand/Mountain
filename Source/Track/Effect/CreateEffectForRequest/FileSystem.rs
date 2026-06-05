pub fn Matches(MethodName:&str) -> bool {
	match MethodName {
		"FileSystem.ReadFile"
		| "FileSystem.WriteFile"
		| "FileSystem.ReadDirectory"
		| "FileSystem.Stat"
		| "FileSystem.CreateDirectory"
		| "FileSystem.Delete"
		| "FileSystem.Rename"
		| "FileSystem.Copy"
		// Aliases folded from FileReadAlias to eliminate duplicate cold-path checks.
		| "openDocument"
		| "readFile"
		| "stat" => true,

		_ => false,
	}
}

use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use CommonLibrary::{
	Environment::Requires::Requires,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{bool_at, str_at, strip_file_uri},
	MappedEffectType::MappedEffect,
};

/// Returns Err with a "resource not found" message for an empty path.
/// Consistent with VS Code's FileSystemProvider contract for empty-path probes.
/// Validated before Environment.Require() in all four read/write handlers.
#[inline]
fn require_non_empty_path(method:&str, path:&str) -> Result<(), String> {
	if path.is_empty() {
		Err(format!("{}: empty path (resource not found)", method))
	} else {
		Ok(())
	}
}

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"FileSystem.ReadFile" => {
			crate::effect!(run_time, {
				let path_str = str_at(&Parameters, 0);

				require_non_empty_path("FileSystem.ReadFile", path_str)?;

				// vscode://schemas-associations/ is a synthetic URI emitted by VS Code's
				// JSON language server to request schema associations from the host.
				// Mountain has no disk-backed file for this; return an empty schema list
				// so the language server does not spin retrying.
				if path_str.starts_with("vscode://schemas-associations/") {
					let payload =
						serde_json::to_vec(&json!({ "schemas": [] })).unwrap_or_else(|_| b"{\"schemas\":[]}".to_vec());

					return Ok(json!(payload));
				}

				let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();

				let path = std::path::PathBuf::from(strip_file_uri(path_str));

				fs_reader
					.ReadFile(&path)
					.await
					.map(|bytes| json!(bytes))
					.map_err(|e| e.to_string())
			})
		},

		"FileSystem.WriteFile" => {
			crate::effect!(run_time, {
				let path_str = str_at(&Parameters, 0);

				require_non_empty_path("FileSystem.WriteFile", path_str)?;

				let path = std::path::PathBuf::from(strip_file_uri(path_str));

				let content = Parameters.get(1).cloned();

				// Only base64-encoded strings are accepted.
				// The Value::Array path (per-byte u64 iteration) was removed:
				//   - O(N) enum-match per byte (200k matches for a 200 KB file)
				//   - filter_map silently drops bytes >127 from signed Int8Array serializers,
				//     causing silent data corruption
				let content_bytes = match content {
					Some(Value::String(s)) => {
						STANDARD
							.decode(&s)
							.map_err(|e| format!("FileSystem.WriteFile: base64 decode failed: {}", e))?
					},
					Some(Value::Array(_)) => {
						return Err(
							"FileSystem.WriteFile: content must be base64-encoded string, not byte array".to_string()
						);
					},
					_ => vec![],
				};

				let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();

				fs_writer
					.WriteFile(&path, content_bytes, true, true)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"FileSystem.ReadDirectory" => {
			crate::effect!(run_time, {
				let path_str = str_at(&Parameters, 0);

				require_non_empty_path("FileSystem.ReadDirectory", path_str)?;

				let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();

				let path = std::path::PathBuf::from(strip_file_uri(path_str));

				fs_reader
					.ReadDirectory(&path)
					.await
					.map(|entries| json!(entries))
					.map_err(|e| e.to_string())
			})
		},

		"FileSystem.Stat" => {
			crate::effect!(run_time, {
				let path_str = str_at(&Parameters, 0);

				// Validate path before Environment.Require() - consistent with
				// ReadFile, WriteFile, ReadDirectory ordering.
				require_non_empty_path("FileSystem.Stat", path_str)?;

				let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();

				let path = std::path::PathBuf::from(strip_file_uri(path_str));

				fs_reader
					.StatFile(&path)
					.await
					.map(|stat| json!(stat))
					.map_err(|e| e.to_string())
			})
		},

		"FileSystem.CreateDirectory" => {
			crate::effect!(run_time, {
				let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();

				let path_str = str_at(&Parameters, 0);

				let path = std::path::PathBuf::from(strip_file_uri(path_str));

				fs_writer
					.CreateDirectory(&path, true)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"FileSystem.Delete" => {
			crate::effect!(run_time, {
				let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();

				let path_str = str_at(&Parameters, 0);

				let path = std::path::PathBuf::from(strip_file_uri(path_str));

				let recursive = bool_at(&Parameters, 1);

				fs_writer
					.Delete(&path, recursive, false)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"FileSystem.Rename" => {
			crate::effect!(run_time, {
				let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();

				let source = str_at(&Parameters, 0);

				let target = str_at(&Parameters, 1);

				fs_writer
					.Rename(
						&std::path::PathBuf::from(strip_file_uri(source)),
						&std::path::PathBuf::from(strip_file_uri(target)),
						true,
					)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		"FileSystem.Copy" => {
			crate::effect!(run_time, {
				let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();

				let source = str_at(&Parameters, 0);

				let target = str_at(&Parameters, 1);

				fs_writer
					.Copy(
						&std::path::PathBuf::from(strip_file_uri(source)),
						&std::path::PathBuf::from(strip_file_uri(target)),
						true,
					)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		_ => None,
	}
}
