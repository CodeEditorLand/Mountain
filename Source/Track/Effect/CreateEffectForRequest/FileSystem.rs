//! # FileSystem Effect (CreateEffectForRequest)
//!
//! Effect constructors for the `FileSystem.*` RPC family. Each handler
//! delegates to the `FileSystemReader` or `FileSystemWriter` provider trait on
//! `MountainEnvironment`. All methods accept `file://` URIs from Cocoon and
//! strip the scheme before passing a native `PathBuf` to the provider.
//!
//! ## Methods handled
//!
//! | Method | Provider | Description |
//! |---|---|---|
//! | `FileSystem.ReadFile` | `FileSystemReader` | Read raw bytes from a file |
//! | `FileSystem.WriteFile` | `FileSystemWriter` | Write bytes to a file |
//! | `FileSystem.ReadDirectory` | `FileSystemReader` | List directory entries |
//! | `FileSystem.Stat` | `FileSystemReader` | Get file metadata |
//! | `FileSystem.CreateDirectory` | `FileSystemWriter` | Create a directory (optionally recursive) |
//! | `FileSystem.Delete` | `FileSystemWriter` | Delete a file or directory |
//! | `FileSystem.Rename` | `FileSystemWriter` | Rename/move a file or directory |
//! | `FileSystem.Copy` | `FileSystemWriter` | Copy a file or directory tree |
//!
//! ## VS Code reference
//!
//! `vs/platform/files/common/fileService.ts`,
//! `vs/base/parts/ipc/common/ipc.net.ts`

use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::STANDARD};
use CommonLibrary::{
	Environment::Requires::Requires,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{bool_at, str_at},
	MappedEffectType::MappedEffect,
};

/// Strip a leading `file://` (or `file:///`) scheme from the incoming path.
/// Cocoon sends full URIs like `file:///<home>/.fiddee/extensions/...`
/// through `FileSystem.ReadFile`/`WriteFile`/`ReadDirectory`; `PathBuf` from
/// such a string treats the scheme literally and every read 404s. Without
/// this the redhat.java activation (and any other extension that uses the
/// gRPC fs.readFile path for its own package.json) fails with "Resource not
/// found: file:///...".
fn StripFileUriScheme(Input:&str) -> &str {
	if let Some(Rest) = Input.strip_prefix("file://") {
		// `file:///Users/...` - the third slash is part of the path, keep it.
		if Rest.starts_with('/') {
			return Rest;
		}

		// `file://localhost/Users/...` - rarely used, but normalise by
		// stripping host-up-to-first-slash. Fall through on failure.
		if let Some(Idx) = Rest.find('/') {
			return &Rest[Idx..];
		}
	}

	Input
}

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"FileSystem.ReadFile" => {
			crate::effect!(run_time, {
				let path_str = str_at(&Parameters, 0);
				// Empty-path guard: extensions occasionally
				// pass `""` to `vscode.workspace.fs.readFile`
				// when probing optional config files. Stock VS
				// Code's FileSystemProvider would return
				// `FileNotFound`; replicating that contract
				// here avoids a panic in `PathBuf::from("")`-
				// rooted FS calls (which can confuse Mountain's
				// path-security guard into emitting a "path
				// outside workspace" rejection that trips the
				// breaker cascade).
				if path_str.is_empty() {
					return Err("FileSystem.ReadFile: empty path (resource not found)".to_string());
				}
				if path_str.starts_with("vscode://schemas-associations/") {
					let payload =
						serde_json::to_vec(&json!({ "schemas": [] })).unwrap_or_else(|_| b"{\"schemas\":[]}".to_vec());
					return Ok(json!(payload));
				}
				let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
				let path = std::path::PathBuf::from(StripFileUriScheme(path_str));
				fs_reader
					.ReadFile(&path)
					.await
					.map(|bytes| json!(bytes))
					.map_err(|e| e.to_string())
			})
		},

		"FileSystem.WriteFile" => {
			crate::effect!(run_time, {
				let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
				let path_str = str_at(&Parameters, 0);
				if path_str.is_empty() {
					return Err("FileSystem.WriteFile: empty path (resource not found)".to_string());
				}
				let path = std::path::PathBuf::from(StripFileUriScheme(path_str));
				let content = Parameters.get(1).cloned();
				let content_bytes = match content {
					Some(Value::Array(arr)) => arr.into_iter().filter_map(|v| v.as_u64().map(|n| n as u8)).collect(),
					Some(Value::String(s)) => STANDARD.decode(&s).unwrap_or_default(),
					_ => vec![],
				};
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
				// Empty-path guard: same contract as ReadFile and
				// Stat. An empty string from an extension probe
				// must return "resource not found" so the
				// LooksLike404 classifier in
				// MountainVinegRPCService downgrades the log level
				// and uses error code -32004 instead of tripping
				// the circuit breaker with a -32000.
				if path_str.is_empty() {
					return Err("FileSystem.ReadDirectory: empty path (resource not found)".to_string());
				}
				let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
				let path = std::path::PathBuf::from(StripFileUriScheme(path_str));
				fs_reader
					.ReadDirectory(&path)
					.await
					.map(|entries| json!(entries))
					.map_err(|e| e.to_string())
			})
		},

		"FileSystem.Stat" => {
			crate::effect!(run_time, {
				let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
				let path_str = str_at(&Parameters, 0);
				// Empty-path guard: same rationale as
				// `FileSystem.ReadFile` above. Returning
				// `not found` matches VS Code's
				// `FileSystemProvider.stat()` contract for
				// probes of paths the extension hasn't
				// validated upstream.
				if path_str.is_empty() {
					return Err("FileSystem.Stat: empty path (resource not found)".to_string());
				}
				let path = std::path::PathBuf::from(StripFileUriScheme(path_str));
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
				let path = std::path::PathBuf::from(StripFileUriScheme(path_str));
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
				let path = std::path::PathBuf::from(StripFileUriScheme(path_str));
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
						&std::path::PathBuf::from(StripFileUriScheme(source)),
						&std::path::PathBuf::from(StripFileUriScheme(target)),
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
						&std::path::PathBuf::from(StripFileUriScheme(source)),
						&std::path::PathBuf::from(StripFileUriScheme(target)),
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
