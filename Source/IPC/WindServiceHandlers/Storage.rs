#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Persistent storage handlers - get, set, delete, keys, bulk operations.

use std::sync::Arc;

use serde_json::{Value, json};
use CommonLibrary::{Environment::Requires::Requires, Storage::StorageProvider::StorageProvider};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Handler for storage get requests
pub async fn StorageGet(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let key = Arguments
		.get(0)
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

/// Handler for storage set requests
pub async fn StorageSet(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let key = Arguments
		.get(0)
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

/// Delete a persistent storage key.
pub async fn StorageDelete(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
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

/// Return all storage keys.
pub async fn StorageKeys(RunTime:Arc<ApplicationRunTime>) -> Result<Value, String> {
	let Storage = RunTime
		.Environment
		.GetAllStorage(true)
		.await
		.map_err(|Error| format!("storage:keys failed: {}", Error))?;

	let Keys:Vec<String> = Storage.as_object().map(|O| O.keys().cloned().collect()).unwrap_or_default();
	Ok(json!(Keys))
}

/// Get all storage items as [key, value] tuples.
/// VS Code's NativeWorkbenchStorageService calls this on initialization.
pub async fn StorageGetItems(RunTime:Arc<ApplicationRunTime>, _Arguments:Vec<Value>) -> Result<Value, String> {
	let provider:Arc<dyn StorageProvider> = RunTime.Environment.Require();

	match provider.GetAllStorage(true).await {
		Ok(State) => {
			// Convert JSON object to array of [key, value] tuples
			if let Some(Obj) = State.as_object() {
				let Tuples:Vec<Value> = Obj
					.iter()
					.map(|(K, V)| {
						let ValStr = match V {
							Value::String(S) => S.clone(),
							_ => V.to_string(),
						};
						json!([K, ValStr])
					})
					.collect();
				Ok(json!(Tuples))
			} else {
				Ok(json!([]))
			}
		},
		Err(_) => Ok(json!([])),
	}
}

/// Update storage items. VS Code sends { insert, delete } where:
/// - insert: Array of [key, value] tuples or Map<string, string>
/// - delete: Array of keys to remove
pub async fn StorageUpdateItems(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let provider:Arc<dyn StorageProvider> = RunTime.Environment.Require();

	if let Some(Updates) = Arguments.get(0).and_then(|V| V.as_object()) {
		// Handle inserts
		if let Some(Inserts) = Updates.get("insert") {
			if let Some(Arr) = Inserts.as_array() {
				for Item in Arr {
					if let Some(Pair) = Item.as_array() {
						if let (Some(Key), Some(Val)) = (Pair.get(0).and_then(|V| V.as_str()), Pair.get(1)) {
							let _ = provider.UpdateStorageValue(true, Key.to_string(), Some(Val.clone())).await;
						}
					}
				}
			} else if let Some(Obj) = Inserts.as_object() {
				for (Key, Val) in Obj {
					let _ = provider.UpdateStorageValue(true, Key.clone(), Some(Val.clone())).await;
				}
			}
		}

		// Handle deletes
		if let Some(Deletes) = Updates.get("delete").and_then(|V| V.as_array()) {
			for Key in Deletes {
				if let Some(K) = Key.as_str() {
					let _ = provider.UpdateStorageValue(true, K.to_string(), None).await;
				}
			}
		}
	}

	Ok(Value::Null)
}
