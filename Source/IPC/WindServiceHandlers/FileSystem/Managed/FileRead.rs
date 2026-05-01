#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Legacy wire method `file:read` (UTF-8 content). Routes via runtime's
//! `FileSystemReader` trait. Not currently wired into dispatch (native
//! variant handles `file:read`); kept for future per-provider routing.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileSystemReader::FileSystemReader};
use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn FileRead(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
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

	dev_log!("vfs-verbose", "read: {} ({} bytes)", path, content.len());
	Ok(json!(content))
}
