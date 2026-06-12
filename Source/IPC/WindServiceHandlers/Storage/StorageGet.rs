//! Read a single value from persistent storage by key. The
//! `false` first arg to `GetStorageValue` selects the
//! workspace-scoped store; `true` would target global storage.

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

	// PERF-16: Boot short-circuit. On first boot VS Code queries storage
	// multiple times synchronously before any data has been written.
	// When the store is empty, every `GetStorageValue` call goes through
	// the full trait dispatch even though the key cannot exist yet.
	// Check the in-memory cache directly; if no entries are present for
	// the workspace scope, return Null immediately.
	if RunTime
		.Environment
		.ApplicationState
		.Configuration
		.MementoWorkspaceStorage
		.lock()
		.is_empty()
	{
		dev_log!("storage", "get: {} (empty-store short-circuit)", key);
		return Ok(Value::Null);
	}

	let provider:Arc<dyn StorageProvider> = RunTime.Environment.Require();

	let value = provider
		.GetStorageValue(false, key)
		.await
		.map_err(|Error| format!("Failed to get storage item: {}", Error))?;

	dev_log!("storage", "get: {}", key);

	// Return JSON null for missing keys. VS Code's storage clients use
	// `value ?? defaultValue` which treats null and undefined identically,
	// so null is the correct sentinel for "key not set".
	Ok(value.unwrap_or(Value::Null))
}
