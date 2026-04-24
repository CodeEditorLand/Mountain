#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Legacy wire method `file:readdir`. Returns raw entries from the
//! runtime's `FileSystemReader`. Not currently wired into dispatch; the
//! native variant serves `file:readdir`.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileSystemReader::FileSystemReader};
use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

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

	dev_log!("vfs-verbose", "readdir_legacy: {} ({} entries)", path, entries.len());
	Ok(json!(entries))
}
