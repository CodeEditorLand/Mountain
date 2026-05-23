//! Wire method `file:readBinary`. Active in dispatch. Routes through the
//! RunTime's `FileSystemReader` so VS Code's `VSBuffer.wrap()` receives
//! bytes that Mountain's provider layer has already normalised.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{Environment::Requires::Requires, FileSystem::FileSystemReader::FileSystemReader};
use serde_json::{Value, json};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let path = Arguments
		.get(0)
		.ok_or("Missing file path".to_string())?
		.as_str()
		.ok_or("File path must be a string".to_string())?;

	let provider:Arc<dyn FileSystemReader> = RunTime.Environment.Require();

	let content = provider
		.ReadFile(&PathBuf::from(path))
		.await
		.map_err(|Error| format!("Failed to read binary file: {}", Error))?;

	dev_log!("vfs-verbose", "readBinary: {} ({} bytes)", path, content.len());

	Ok(json!(content))
}
