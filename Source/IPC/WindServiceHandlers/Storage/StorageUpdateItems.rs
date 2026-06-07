//! Bulk insert + delete in one round-trip. VS Code's
//! `IndexedDBStorageDatabase` batches every write through this
//! shape: `{ insert: [[key,value], …] | { key: value }, delete: [keys…] }`.
//! Both insert encodings (array-of-pairs and object-map) accepted.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Storage::StorageProvider::StorageProvider};

use serde_json::Value;

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {

	let provider:Arc<dyn StorageProvider> = RunTime.Environment.Require();

	if let Some(Updates) = Arguments.first().and_then(|V| V.as_object()) {
		if let Some(Inserts) = Updates.get("insert") {
			if let Some(Arr) = Inserts.as_array() {
				for Item in Arr {
					if let Some(Pair) = Item.as_array()

						&& let (Some(Key), Some(Val)) = (Pair.first().and_then(|V| V.as_str()), Pair.get(1))

					{
						let _ = provider.UpdateStorageValue(true, Key.to_string(), Some(Val.clone())).await;
					}
				}
			} else if let Some(Obj) = Inserts.as_object() {
				for (Key, Val) in Obj {
					let _ = provider.UpdateStorageValue(true, Key.clone(), Some(Val.clone())).await;
				}
			}
		}

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
