//! Legacy wire method `file:mkdir`. Recursive by default
//! (`Arguments[1]` honoured when supplied as bool).

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::FileSystemWriter::FileSystemWriter,
};
use serde_json::Value;

use crate::{
	IPC::WindServiceHandlers::Utilities::JsonValueHelpers::ArgBoolTrue,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = Arguments
		.Get(0)
		.ok_or("Missing directory path".to_string())?
		.as_str()
		.ok_or("Directory path must be a string".to_string())?;

	let Recursive = ArgBoolTrue(&Arguments, 1);

	let Provider:Arc<dyn FileSystemWriter> = RunTime.Environment.Require();

	provider
		.CreateDirectory(&PathBuf::from(path), recursive)
		.await
		.map_err(|E:CommonError| format!("Failed to create directory: {}", e))?;

	dev_log!("vfs-verbose", "mkdir: {} (recursive: {})", path, recursive);

	Ok(Value::Null)
}
