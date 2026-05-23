//! Legacy wire method `file:write` (UTF-8 content). Routes via RunTime's
//! `FileSystemWriter` trait. Not currently wired into dispatch.

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

	let content = Arguments
		.get(1)
		.ok_or("Missing file content".to_string())?
		.as_str()
		.ok_or("File content must be a string".to_string())?;

	let provider:Arc<dyn FileSystemWriter> = RunTime.Environment.Require();

	provider
		.WriteFile(&PathBuf::from(path), content.as_bytes().to_vec(), true, true)
		.await
		.map_err(|e:CommonError| format!("Failed to write file: {}", e))?;

	dev_log!("vfs-verbose", "written: {} ({} bytes)", path, content.len());

	Ok(Value::Null)
}
