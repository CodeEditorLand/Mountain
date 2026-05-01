#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Legacy wire method `file:exists`. Returns bool via `StatFile` probe.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileSystemReader::FileSystemReader};
use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn FileExists(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let path = Arguments
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemReader> = RunTime.Environment.Require();

	let exists = provider.StatFile(&PathBuf::from(path)).await.is_ok();

	dev_log!("vfs-verbose", "exists: {} = {}", path, exists);
	Ok(json!(exists))
}
