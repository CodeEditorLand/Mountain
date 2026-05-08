#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Legacy wire method `file:move`. Non-overwriting rename.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::FileSystemWriter::FileSystemWriter,
};
use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn FileMove(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let source = Arguments
		.get(0)
		.ok_or("Missing source path".to_string())?
		.as_str()
		.ok_or("Source path must be a string".to_string())?;

	let destination = Arguments
		.get(1)
		.ok_or("Missing destination path".to_string())?
		.as_str()
		.ok_or("Destination path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemWriter> = RunTime.Environment.Require();

	provider
		.Rename(&PathBuf::from(source), &PathBuf::from(destination), false)
		.await
		.map_err(|e:CommonError| format!("Failed to move file: {} -> {}", source, destination))?;

	dev_log!("vfs-verbose", "moved: {} -> {}", source, destination);

	Ok(Value::Null)
}
