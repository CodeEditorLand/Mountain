//! Legacy wire method `file:delete`. Non-recursive, non-trash.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::FileSystemWriter::FileSystemWriter,
};
use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let path = Arguments
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemWriter> = RunTime.Environment.Require();

	provider
		.Delete(&PathBuf::from(path), false, false)
		.await
		.map_err(|e:CommonError| format!("Failed to delete file: {}", e))?;

	dev_log!("vfs-verbose", "deleted: {}", path);

	Ok(Value::Null)
}
