#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(
	MethodName:&str,
	Parameters:Value,
) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Languages.GetAll" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let scanned = run_time
							.Environment
							.ApplicationState
							.Extension
							.ScannedExtensions
							.ScannedExtensions
							.clone();
						let Guard = match scanned.lock() {
							Ok(g) => g,
							Err(error) => {
								return Err(format!(
									"Languages.GetAll: scanned-extensions lock poisoned: {}",
									error
								));
							},
						};

						let mut merged:HashMap<String, serde_json::Map<String, Value>> =
							HashMap::new();
						for Dto in Guard.values() {
							let Contributes = match Dto.Contributes.as_ref() {
								Some(c) => c,
								None => continue,
							};
							let Languages =
								Contributes.get("languages").and_then(Value::as_array);
							let Some(Languages) = Languages else { continue };
							for Entry in Languages {
								let Id = match Entry.get("id").and_then(Value::as_str) {
									Some(id) if !id.is_empty() => id.to_string(),
									_ => continue,
								};
								let Existing = merged.entry(Id.clone()).or_insert_with(|| {
									let mut seed = serde_json::Map::new();
									seed.insert("id".to_string(), json!(Id));
									seed.insert("aliases".to_string(), json!([]));
									seed.insert("extensions".to_string(), json!([]));
									seed.insert("filenames".to_string(), json!([]));
									seed.insert("filenamePatterns".to_string(), json!([]));
									seed.insert("mimetypes".to_string(), json!([]));
									seed.insert("configuration".to_string(), Value::Null);
									seed
								});
								let merge_array = |target:&mut serde_json::Map<String, Value>,
								                   key:&str,
								                   incoming:&Value| {
									let Some(incoming_arr) =
										incoming.get(key).and_then(Value::as_array)
									else {
										return;
									};
									let bucket = target
										.entry(key.to_string())
										.or_insert_with(|| json!([]));
									if let Some(bucket_arr) = bucket.as_array_mut() {
										for v in incoming_arr {
											if !bucket_arr.iter().any(|e| e == v) {
												bucket_arr.push(v.clone());
											}
										}
									}
								};
								merge_array(Existing, "aliases", Entry);
								merge_array(Existing, "extensions", Entry);
								merge_array(Existing, "filenames", Entry);
								merge_array(Existing, "filenamePatterns", Entry);
								merge_array(Existing, "mimetypes", Entry);
								if Existing.get("configuration").map(Value::is_null).unwrap_or(true) {
									if let Some(cfg) = Entry.get("configuration") {
										Existing.insert("configuration".to_string(), cfg.clone());
									}
								}
							}
						}
						drop(Guard);

						let result:Vec<Value> =
							merged.into_values().map(Value::Object).collect();
						dev_log!("ipc", "[Languages.GetAll] returning {} languages", result.len());
						Ok(json!(result))
					})
				};
			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
