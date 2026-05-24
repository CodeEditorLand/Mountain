//! Cocoon legacy aliases: `openDocument`, `readFile`, `stat` - short-hand
//! routes used by Cocoon's Effect-TS Workspace + FileSystem services before
//! the canonical `FileSystem.*` naming was established. Backed by the same
//! `FileSystemReader` provider.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileSystemReader::FileSystemReader};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{
	CreateEffectForRequest::Utilities::Params::{StrObjOrPos, StripFileUri},
	MappedEffectType::MappedEffect,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"openDocument" | "readFile" | "stat" => {
			let MethodNameOwned = MethodName.to_string();

			crate::effect!(RunTime, {
				let FsReader:Arc<dyn FileSystemReader> = RunTime.Environment.Require();
				let Path = {
					let s = StrObjOrPos(&Parameters, "uri", 0);
					if s.is_empty() { StrObjOrPos(&Parameters, "path", 0) } else { s }
				}
				.to_string();
				// Empty-path guard: matches the FileSystem.* contract so that
				// the LooksLike404 classifier in MountainVinegRPCService
				// downgrades the log level and uses error code -32004 rather
				// than tripping the circuit breaker with a -32000.
				if Path.is_empty() {
					return Err(format!("{}: empty path (resource not found)", MethodNameOwned));
				}
				let PathBuf_ = std::path::PathBuf::from(StripFileUri(&Path));
				match MethodNameOwned.as_str() {
					"stat" => {
						FsReader
							.StatFile(&PathBuf_)
							.await
							.map(|S| serde_json::to_value(S).unwrap_or(Value::Null))
							.map_err(|E| e.to_string())
					},
					"readFile" | "openDocument" => {
						FsReader
							.ReadFile(&PathBuf_)
							.await
							.map(|Bytes| {
								let Text = String::from_utf8(Bytes).unwrap_or_default();
								json!({ "uri": Path, "text": Text })
							})
							.map_err(|E| e.to_string())
					},
					_ => Ok(Value::Null),
				}
			})
		},

		_ => None,
	}
}
