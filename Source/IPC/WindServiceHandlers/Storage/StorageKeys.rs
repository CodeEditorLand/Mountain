//! Return every key in global storage as a JSON array. Used by
//! Wind's storage-debug surfaces and by extensions iterating
//! the global storage namespace.

use std::sync::Arc;

use CommonLibrary::Storage::StorageProvider::StorageProvider;
use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Storage = RunTime
		.Environment
		.GetAllStorage(true)
		.await
		.map_err(|Error| format!("storage:keys failed: {}", Error))?;

	let Keys:Vec<String> = Storage.as_object().map(|O| O.keys().cloned().collect()).unwrap_or_default();

	Ok(json!(Keys))
}
