#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Cocoon legacy aliases: `openDocument`, `readFile`, `stat` - short-hand
//! routes used by Cocoon's Effect-TS Workspace + FileSystem services before
//! the canonical `FileSystem.*` naming was established. Backed by the same
//! `FileSystemReader` provider.

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileSystemReader::FileSystemReader};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"openDocument" | "readFile" | "stat" => {
			let MethodNameOwned = MethodName.to_string();

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let fs_reader:Arc<dyn FileSystemReader> = run_time.Environment.Require();
						let Path = if let Some(Object) = Parameters.as_object() {
							Object
								.get("uri")
								.or_else(|| Object.get("path"))
								.and_then(Value::as_str)
								.unwrap_or("")
								.to_string()
						} else {
							Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string()
						};
						let PathBuf_ = std::path::PathBuf::from(&Path);
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
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
