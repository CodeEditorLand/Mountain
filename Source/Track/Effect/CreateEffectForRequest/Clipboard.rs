#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Clipboard.Read" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {

					Box::pin(async move {
						let result = tokio::task::spawn_blocking(|| -> Result<String, String> {
							let mut Clipboard = arboard::Clipboard::new().map_err(|e| e.to_string())?;
							Clipboard.get_text().map_err(|e| e.to_string())
						})
						.await
						.map_err(|e| format!("Clipboard.Read join error: {}", e))?;
						match result {
							Ok(text) => Ok(json!(text)),
							Err(e) => {
								if e.contains("empty") || e.contains("Empty") {
									Ok(json!(""))
								} else {
									dev_log!("ipc", "warn: [Clipboard.Read] {}", e);
									Err(e)
								}
							},
						}
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"Clipboard.Write" => {
			let effect =
				move |_run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {

					Box::pin(async move {
						let text =
							Parameters.get(0).and_then(Value::as_str).unwrap_or("").to_string();
						let text_len = text.len();
						let result = tokio::task::spawn_blocking(move || -> Result<(), String> {
							let mut Clipboard =
								arboard::Clipboard::new().map_err(|e| e.to_string())?;
							Clipboard.set_text(text).map_err(|e| e.to_string())
						})
						.await
						.map_err(|e| format!("Clipboard.Write join error: {}", e))?;
						result.map(|()| {
							dev_log!("ipc", "[Clipboard.Write] wrote {} byte(s)", text_len);
							json!(null)
						})
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
