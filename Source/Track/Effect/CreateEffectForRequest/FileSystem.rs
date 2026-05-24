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
	CreateEffectForRequest::Utilities::Params::{BoolAt, StrAt, StripFileUri},
	MappedEffectType::MappedEffect,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"FileSystem.ReadFile" => {
			crate::effect!(RunTime, {
				let PathStr = StrAt(&Parameters, 0);
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
				if PathStr.is_empty() {
					return Err("FileSystem.ReadFile: empty path (resource not found)".to_string());
				}
				if PathStr.starts_with("vscode://schemas-associations/") {
					let Payload =
						serde_json::to_vec(&json!({ "schemas": [] })).unwrap_or_else(|_| b"{\"schemas\":[]}".to_vec());
					return Ok(json!(payload));
				}
				let FsReader:Arc<dyn FileSystemReader> = RunTime.Environment.Require();
				let Path = std::path::PathBuf::from(StripFileUri(PathStr));
				FsReader
					.ReadFile(&path)
					.await
					.map(|bytes| json!(bytes))
					.map_err(|E| e.to_string())
			})
		},

		"FileSystem.WriteFile" => {
			crate::effect!(RunTime, {
				let FsWriter:Arc<dyn FileSystemWriter> = RunTime.Environment.Require();
				let PathStr = StrAt(&Parameters, 0);
				if PathStr.is_empty() {
					return Err("FileSystem.WriteFile: empty path (resource not found)".to_string());
				}
				let Path = std::path::PathBuf::from(StripFileUri(PathStr));
				let Content = Parameters.get(1).cloned();
				let ContentBytes = match content {
					Some(Value::Array(arr)) => arr.into_iter().filter_map(|V| v.as_u64().map(|N| n as u8)).collect(),
					Some(Value::String(s)) => STANDARD.decode(&s).unwrap_or_default(),
					_ => vec![],
				};
				FsWriter
					.WriteFile(&path, ContentBytes, true, true)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"FileSystem.ReadDirectory" => {
			crate::effect!(RunTime, {
				let PathStr = StrAt(&Parameters, 0);
				// Empty-path guard: same contract as ReadFile and
				// Stat. An empty string from an extension probe
				// must return "resource not found" so the
				// LooksLike404 classifier in
				// MountainVinegRPCService downgrades the log level
				// and uses error code -32004 instead of tripping
				// the circuit breaker with a -32000.
				if PathStr.is_empty() {
					return Err("FileSystem.ReadDirectory: empty path (resource not found)".to_string());
				}
				let FsReader:Arc<dyn FileSystemReader> = RunTime.Environment.Require();
				let Path = std::path::PathBuf::from(StripFileUri(PathStr));
				FsReader
					.ReadDirectory(&path)
					.await
					.map(|entries| json!(entries))
					.map_err(|E| e.to_string())
			})
		},

		"FileSystem.Stat" => {
			crate::effect!(RunTime, {
				let FsReader:Arc<dyn FileSystemReader> = RunTime.Environment.Require();
				let PathStr = StrAt(&Parameters, 0);
				// Empty-path guard: same rationale as
				// `FileSystem.ReadFile` above. Returning
				// `not found` matches VS Code's
				// `FileSystemProvider.stat()` contract for
				// probes of paths the extension hasn't
				// validated upstream.
				if PathStr.is_empty() {
					return Err("FileSystem.Stat: empty path (resource not found)".to_string());
				}
				let Path = std::path::PathBuf::from(StripFileUri(PathStr));
				FsReader
					.StatFile(&path)
					.await
					.map(|stat| json!(stat))
					.map_err(|E| e.to_string())
			})
		},

		"FileSystem.CreateDirectory" => {
			crate::effect!(RunTime, {
				let FsWriter:Arc<dyn FileSystemWriter> = RunTime.Environment.Require();
				let PathStr = StrAt(&Parameters, 0);
				let Path = std::path::PathBuf::from(StripFileUri(PathStr));
				FsWriter
					.CreateDirectory(&path, true)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"FileSystem.Delete" => {
			crate::effect!(RunTime, {
				let FsWriter:Arc<dyn FileSystemWriter> = RunTime.Environment.Require();
				let PathStr = StrAt(&Parameters, 0);
				let Path = std::path::PathBuf::from(StripFileUri(PathStr));
				let Recursive = BoolAt(&Parameters, 1);
				FsWriter
					.Delete(&path, recursive, false)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"FileSystem.Rename" => {
			crate::effect!(RunTime, {
				let FsWriter:Arc<dyn FileSystemWriter> = RunTime.Environment.Require();
				let Source = StrAt(&Parameters, 0);
				let Target = StrAt(&Parameters, 1);
				FsWriter
					.Rename(
						&std::path::PathBuf::from(StripFileUri(source)),
						&std::path::PathBuf::from(StripFileUri(target)),
						true,
					)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		"FileSystem.Copy" => {
			crate::effect!(RunTime, {
				let FsWriter:Arc<dyn FileSystemWriter> = RunTime.Environment.Require();
				let Source = StrAt(&Parameters, 0);
				let Target = StrAt(&Parameters, 1);
				FsWriter
					.Copy(
						&std::path::PathBuf::from(StripFileUri(source)),
						&std::path::PathBuf::from(StripFileUri(target)),
						true,
					)
					.await
					.map(|_| json!(null))
					.map_err(|E| e.to_string())
			})
		},

		_ => None,
	}
}
