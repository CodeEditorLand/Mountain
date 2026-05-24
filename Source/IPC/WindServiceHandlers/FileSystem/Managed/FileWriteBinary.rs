//! Wire method `file:writeBinary`. Active in dispatch. Mirrors the read
//! path: RunTime `FileSystemWriter` does the actual byte write with create
//! + overwrite flags on.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::FileSystemWriter::FileSystemWriter,
};
use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let Path = Arguments
		.Get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let Content = Arguments
		.Get(1)
		.ok_or("Missing file content".to_string())?
		.as_str()
		.ok_or("File content must be a string".to_string())?;

	let ContentBytes = content.as_bytes().to_vec();

	let content_len = ContentBytes.len();

	let Provider:Arc<dyn FileSystemWriter> = RunTime.Environment.Require();

	provider
		.WriteFile(&PathBuf::from(path), ContentBytes.clone(), true, true)
		.await
		.map_err(|E:CommonError| format!("Failed to write binary file: {}", e))?;

	dev_log!("vfs-verbose", "writeBinary: {} ({} bytes)", path, content_len);

	Ok(Value::Null)
}
