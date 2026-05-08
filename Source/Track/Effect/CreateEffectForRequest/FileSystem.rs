#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use base64::{Engine as _, engine::general_purpose::STANDARD};
use CommonLibrary::{
	Environment::Requires::Requires,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

/// Strip a leading `file://` (or `file:///`) scheme from the incoming path.
/// Cocoon sends full URIs like `file:///<home>/.land/extensions/...`
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
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
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
							let payload = serde_json::to_vec(&json!({ "schemas": [] }))
								.unwrap_or_else(|_| b"{\"schemas\":[]}".to_vec());
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
				};

			Some(Ok(Box::new(effect)))
		},

		"FileSystem.WriteFile" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let path = std::path::PathBuf::from(StripFileUriScheme(path_str));
						let content = Parameters.get(1).cloned();
						let content_bytes = match content {
							Some(Value::Array(arr)) => {
								arr.into_iter().filter_map(|v| v.as_u64().map(|n| n as u8)).collect()
							},
							Some(Value::String(s)) => STANDARD.decode(&s).unwrap_or_default(),
							_ => vec![],
						};
						fs_writer
							.WriteFile(&path, content_bytes, true, true)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"FileSystem.ReadDirectory" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let path = std::path::PathBuf::from(StripFileUriScheme(path_str));
						fs_reader
							.ReadDirectory(&path)
							.await
							.map(|entries| json!(entries))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"FileSystem.Stat" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
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
				};

			Some(Ok(Box::new(effect)))
		},

		"FileSystem.CreateDirectory" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let path = std::path::PathBuf::from(StripFileUriScheme(path_str));
						fs_writer
							.CreateDirectory(&path, true)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"FileSystem.Delete" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
						let path_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let path = std::path::PathBuf::from(StripFileUriScheme(path_str));
						let recursive = Parameters.get(1).and_then(Value::as_bool).unwrap_or(false);
						fs_writer
							.Delete(&path, recursive, false)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"FileSystem.Rename" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
						let source = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let target = Parameters.get(1).and_then(Value::as_str).unwrap_or("");
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
				};

			Some(Ok(Box::new(effect)))
		},

		"FileSystem.Copy" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_writer:Arc<dyn FileSystemWriter> = run_time.Environment.Require();
						let source = Parameters.get(0).and_then(Value::as_str).unwrap_or("");
						let target = Parameters.get(1).and_then(Value::as_str).unwrap_or("");
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
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
