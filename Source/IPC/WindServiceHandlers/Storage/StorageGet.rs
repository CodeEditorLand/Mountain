#![allow(non_snake_case)]

//! Read a single value from persistent storage by key. The
//! `false` first arg to `GetStorageValue` selects the
//! workspace-scoped store; `true` would target global storage.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Storage::StorageProvider::StorageProvider};

use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn StorageGet(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let key = Arguments
		.first()
		.ok_or("Missing storage key".to_string())?
		.as_str()
		.ok_or("Storage key must be a string".to_string())?;

	let provider:Arc<dyn StorageProvider> = RunTime.Environment.Require();

	let value = provider
		.GetStorageValue(false, key)
		.await
		.map_err(|Error| format!("Failed to get storage item: {}", Error))?;

	dev_log!("storage", "get: {}", key);

	Ok(value.unwrap_or(Value::Null))
}
