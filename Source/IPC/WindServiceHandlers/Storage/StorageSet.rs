//! Write a single value to workspace-scoped storage. Atomic per
//! key - concurrent set/get against the same key serialise
//! through the StorageProvider's lock.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Storage::StorageProvider::StorageProvider};

use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let key = Arguments
		.first()
		.ok_or("Missing storage key".to_string())?
		.as_str()
		.ok_or("Storage key must be a string".to_string())?;

	let value = Arguments.get(1).ok_or("Missing storage value".to_string())?.clone();

	let provider:Arc<dyn StorageProvider> = RunTime.Environment.Require();

	provider
		.UpdateStorageValue(false, key.to_string(), Some(value))
		.await
		.map_err(|Error| format!("Failed to set storage item: {}", Error))?;

	dev_log!("storage", "set: {}", key);

	Ok(Value::Null)
}
