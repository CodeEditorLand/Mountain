pub fn Matches(MethodName:&str) -> bool {
	match MethodName {
		"Storage.Get" | "Storage.Set" => true,

		_ => false,
	}
}

use std::sync::Arc;

use CommonLibrary::{Environment::Requires::Requires, Storage::StorageProvider::StorageProvider};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::Track::Effect::{CreateEffectForRequest::Utilities::Params::string_at, MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Storage.Get" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn StorageProvider> = run_time.Environment.Require();

				let key = string_at(&Parameters, 0);

				provider
					.GetStorageValue(false, &key)
					.await
					.map(|opt_val| json!(opt_val))
					.map_err(|e| e.to_string())
			})
		},

		"Storage.Set" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn StorageProvider> = run_time.Environment.Require();

				let key = string_at(&Parameters, 0);

				let value = Parameters.get(1).cloned();

				provider
					.UpdateStorageValue(false, key, value)
					.await
					.map(|_| json!(null))
					.map_err(|e| e.to_string())
			})
		},

		// Bulk-read all key/value pairs as `[[key, value]]` tuples.
		// Cocoon's Memento calls this once at boot to hydrate its cache.
		// Without this arm the call fell through to "Unknown method" and
		// every extension's persisted state was lost on each session.
		"Storage.GetItems" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn StorageProvider> = run_time.Environment.Require();

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
			})
		},

		_ => None,
	}
}
