#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Legacy wire method `file:readdir`. Returns raw entries from the
//! RunTime's `FileSystemReader`. Not currently wired into dispatch; the
//! native variant serves `file:readdir`.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileSystemReader::FileSystemReader};

use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn FileReaddir(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let path = Arguments
		.get(0)
		.ok_or("Missing directory path".to_string())?
		.as_str()
		.ok_or("Directory path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemReader> = RunTime.Environment.Require();

	let entries = provider
		.ReadDirectory(&PathBuf::from(path))
		.await
		.map_err(|Error| format!("Failed to read directory: {}", Error))?;

	dev_log!("vfs-verbose", "readdir_legacy: {} ({} entries)", path, entries.len());

	Ok(json!(entries))
}
