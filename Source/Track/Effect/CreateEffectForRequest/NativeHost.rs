
use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"NativeHost.OpenExternal" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {

					Box::pin(async move {
						let uri = Parameters
							.get(0)
							.and_then(Value::as_str)
							.unwrap_or("")
							.to_string();
						let lower = uri.to_ascii_lowercase();
						const BlockedSchemes:&[&str] =
							&["javascript:", "data:", "vbscript:", "file:"];
						for scheme in BlockedSchemes {
							if lower.starts_with(scheme) {
								dev_log!(
									"ipc",

									"warn: [NativeHost.OpenExternal] rejected scheme '{}': {}",

									scheme,

									uri
								);
								return Err(format!(
									"NativeHost.OpenExternal: scheme '{}' is not allowed",

									scheme
								));
							}
						}
						if uri.is_empty() {
							return Err("NativeHost.OpenExternal: empty URI".to_string());
						}
						let uri_owned = uri.clone();
						let result =
							tokio::task::spawn_blocking(move || open::that_detached(uri_owned))
								.await
								.map_err(|e| format!("NativeHost.OpenExternal join error: {}", e))?;
						match result {
							Ok(()) => {
								dev_log!("ipc", "[NativeHost.OpenExternal] opened {}", uri);
								Ok(json!(true))
							},
							Err(e) => {
								dev_log!(
									"ipc",

									"warn: [NativeHost.OpenExternal] failed uri={} error={}",

									uri,

									e
								);
								Err(e.to_string())
							},
						}
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
