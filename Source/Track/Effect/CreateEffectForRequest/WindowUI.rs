#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

//! Window-namespace UI commands from Cocoon's window shim. These emit Tauri
//! events to Sky and return immediately (no reply channel wired yet).

use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(
	MethodName:&str,
	Parameters:Value,
) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Window.ShowMessage" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let AppHandle = run_time.Environment.ApplicationHandle.clone();
						let Payload = if Parameters.is_array() {
							Parameters.get(0).cloned().unwrap_or_default()
						} else {
							Parameters
						};
						let Id = format!(
							"notification-{}",
							std::time::SystemTime::now()
								.duration_since(std::time::UNIX_EPOCH)
								.map(|D| D.as_millis())
								.unwrap_or(0)
						);
						let Message =
							Payload.get("message").and_then(Value::as_str).unwrap_or("").to_string();
						let Level =
							Payload.get("level").and_then(Value::as_str).unwrap_or("info").to_string();
						let Items = Payload.get("items").cloned().unwrap_or(json!([]));
						let Options = Payload.get("options").cloned().unwrap_or(json!({}));
						if let Err(Error) = AppHandle.emit(
							"sky://notification/show",
							json!({
								"id": Id,
								"message": Message,
								"severity": Level,
								"actions": Items,
								"options": Options,
							}),
						) {
							dev_log!(
								"notification",
								"warn: [Window.ShowMessage] sky://notification/show emit failed: {}",
								Error
							);
						}
						Ok(Value::Null)
					})
				};
			Some(Ok(Box::new(effect)))
		},

		"Window.ShowQuickPick"
		| "Window.ShowInputBox"
		| "Window.ShowOpenDialog"
		| "Window.ShowSaveDialog" => {
			let MethodNameOwned = MethodName.to_string();
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;
						let Args =
							if Parameters.is_array() { Parameters } else { json!([Parameters]) };
						let Channel = match MethodNameOwned.as_str() {
							"Window.ShowQuickPick" => "sky://quickpick/show",
							"Window.ShowInputBox" => "sky://input-box/show",
							"Window.ShowOpenDialog" => "sky://dialog/open",
							"Window.ShowSaveDialog" => "sky://dialog/save",
							_ => "sky://quickpick/show",
						};
						let AppHandle = run_time.Environment.ApplicationHandle.clone();
						let Nonce = format!(
							"ui-{}",
							std::time::SystemTime::now()
								.duration_since(std::time::UNIX_EPOCH)
								.map(|D| D.as_nanos())
								.unwrap_or(0)
						);
						if let Err(Error) =
							AppHandle.emit(Channel, json!({ "nonce": Nonce, "args": Args }))
						{
							dev_log!(
								"ipc",
								"warn: [{}] {} emit failed: {}",
								MethodNameOwned,
								Channel,
								Error
							);
						}
						Ok(Value::Null)
					})
				};
			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
