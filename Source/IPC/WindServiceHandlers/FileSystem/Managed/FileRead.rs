#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Legacy wire method `file:read` (UTF-8 content). Routes via RunTime's
//! `FileSystemReader` trait. Not currently wired into dispatch (native
//! variant handles `file:read`); kept for future per-provider routing.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileSystemReader::FileSystemReader};
use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let path = Arguments
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemReader> = RunTime.Environment.Require();

	let content = provider
		.ReadFile(&PathBuf::from(path))
		.await
		.map_err(|Error| format!("Failed to read file: {}", Error))?;

	dev_log!("vfs-verbose", "read: {} ({} bytes)", path, content.len());

	Ok(json!(content))
}
