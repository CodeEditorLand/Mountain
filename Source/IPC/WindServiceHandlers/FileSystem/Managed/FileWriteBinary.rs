#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

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

pub async fn FileWriteBinary(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

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

	let content_bytes = content.as_bytes().to_vec();

	let content_len = content_bytes.len();

	let provider:Arc<dyn FileSystemWriter> = RunTime.Environment.Require();

	provider
		.WriteFile(&PathBuf::from(path), content_bytes.clone(), true, true)
		.await
		.map_err(|e:CommonError| format!("Failed to write binary file: {}", e))?;

	dev_log!("vfs-verbose", "writeBinary: {} ({} bytes)", path, content_len);

	Ok(Value::Null)
}
