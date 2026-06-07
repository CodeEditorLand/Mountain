//! Delete a key from global storage. The `true` first arg to
//! `UpdateStorageValue` targets the global (cross-workspace)
//! store; pairs with `StorageKeys` / `StorageGetItems` which
//! also read from global.

use std::sync::Arc;

use CommonLibrary::Storage::StorageProvider::StorageProvider;

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let Key = Arguments
		.first()
		.and_then(|V| V.as_str())
		.ok_or("storage:delete requires key as first argument".to_string())?
		.to_string();

	RunTime
		.Environment
		.UpdateStorageValue(true, Key, None)
		.await
		.map_err(|Error| format!("storage:delete failed: {}", Error))?;

	Ok(Value::Null)
}
