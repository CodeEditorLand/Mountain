#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Wire method `file:readBinary`. Active in dispatch. Routes through the
//! runtime's `FileSystemReader` so VS Code's `VSBuffer.wrap()` receives
//! bytes that Mountain's provider layer has already normalised.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileSystemReader::FileSystemReader};
use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn handle_file_read_binary(
	runtime:Arc<ApplicationRunTime>,
	args:Vec<Value>,
) -> Result<Value, String> {
	let path = args
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemReader> = runtime.Environment.Require();

	let content = provider
		.ReadFile(&PathBuf::from(path))
		.await
		.map_err(|e| format!("Failed to read binary file: {}", e))?;

	dev_log!("vfs", "readBinary: {} ({} bytes)", path, content.len());
	Ok(json!(content))
}
