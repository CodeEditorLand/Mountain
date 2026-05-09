#![allow(non_snake_case)]

//! Bulk-read every key/value pair as `[key, value]` tuples.
//! VS Code's `NativeWorkbenchStorageService` calls this exactly
//! once at boot to hydrate its in-memory cache. Stringifies
//! non-string values for wire-shape compatibility with the
//! upstream `StorageDatabase` contract.

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Storage::StorageProvider::StorageProvider};
use serde_json::{Value, json};

use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

pub async fn StorageGetItems(RunTime:Arc<ApplicationRunTime>, _Arguments:Vec<Value>) -> Result<Value, String> {
	let provider:Arc<dyn StorageProvider> = RunTime.Environment.Require();

	match provider.GetAllStorage(true).await {
		Ok(State) => {
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
