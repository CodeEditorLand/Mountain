/// matches.
pub fn Matches(MethodName:&str) -> bool {
	match MethodName {
		"openDocument" | "readFile" | "stat" => true,

		_ => false,
	}
}

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileSystemReader::FileSystemReader};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{str_obj_or_pos, strip_file_uri},
	MappedEffectType::MappedEffect,
};

/// Creates effect.
pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"openDocument" | "readFile" | "stat" => {
			let MethodNameOwned = MethodName.to_string();

			crate::effect!(run_time, {
				let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();

				let Path = {
					let s = str_obj_or_pos(&Parameters, "uri", 0);

					match s.is_empty() {
						false => s,
						true => str_obj_or_pos(&Parameters, "path", 0),
					}
				}
				.to_string();

				// Empty-path guard: matches the FileSystem.* contract so that
				// the LooksLike404 classifier in MountainVinegRPCService
				// downgrades the log level and uses error code -32004 rather
				// than tripping the circuit breaker with a -32000.
				match Path.is_empty() {
					true => return Err(format!("{}: empty path (resource not found)", MethodNameOwned)),
					false => {},
				}

				let PathBuf_ = std::path::PathBuf::from(strip_file_uri(&Path));

				match MethodNameOwned.as_str() {
					"stat" => {
						fs_reader
							.StatFile(&PathBuf_)
							.await
							.map(|S| serde_json::to_value(S).unwrap_or(Value::Null))
							.map_err(|e| e.to_string())
					},
					"readFile" | "openDocument" => {
						fs_reader
							.ReadFile(&PathBuf_)
							.await
							.map(|Bytes| {
								let Text = String::from_utf8(Bytes).unwrap_or_default();

								json!({ "uri": Path, "text": Text })
							})
							.map_err(|e| e.to_string())
					},
					_ => Ok(Value::Null),
				}
			})
		},

		_ => None,
	}
}
