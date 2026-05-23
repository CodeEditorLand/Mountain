//! Window-namespace UI commands from Cocoon's window shim.
//! ShowMessage is fire-and-forget (no selection reply needed).
//! ShowQuickPick / ShowInputBox / ShowOpenDialog / ShowSaveDialog block on
//! a oneshot channel that is resolved by the frontend via ResolveUIRequest.

use std::{future::Future, pin::Pin, sync::Arc};

use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	ApplicationState::State::ApplicationState::ApplicationState,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::MappedEffectType::MappedEffect,
	dev_log,
};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
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
						let Message = Payload.get("message").and_then(Value::as_str).unwrap_or("").to_string();
						let Level = Payload.get("level").and_then(Value::as_str).unwrap_or("info").to_string();
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

		"Window.ShowQuickPick" | "Window.ShowInputBox" | "Window.ShowOpenDialog" | "Window.ShowSaveDialog" => {
			let MethodNameOwned = MethodName.to_string();

			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						use tauri::Emitter;

						let Args = if Parameters.is_array() { Parameters } else { json!([Parameters]) };

						let Channel = match MethodNameOwned.as_str() {
							"Window.ShowQuickPick" => "sky://quickpick/show",
							"Window.ShowInputBox" => "sky://input-box/show",
							"Window.ShowOpenDialog" => "sky://dialog/open",
							"Window.ShowSaveDialog" => "sky://dialog/save",
							_ => "sky://quickpick/show",
						};

						use std::sync::atomic::{AtomicU64, Ordering as AO};

						static UI_SEQ:AtomicU64 = AtomicU64::new(1);

						let Nonce = format!("ui-{}", UI_SEQ.fetch_add(1, AO::Relaxed));

						// Register the reply channel before emitting so the
						// frontend can never race-resolve before we are waiting.
						let (tx, rx) = tokio::sync::oneshot::channel();
						run_time.Environment.ApplicationState.UI.AddPendingRequest(Nonce.clone(), tx);

						let AppHandle = run_time.Environment.ApplicationHandle.clone();
						if let Err(Error) = AppHandle.emit(Channel, json!({ "nonce": Nonce, "args": Args })) {
							// Emit failed -- remove the dangling sender so the map
							// does not grow unboundedly on repeated failures.
							run_time.Environment.ApplicationState.UI.RemovePendingRequest(&Nonce.clone());
							dev_log!("ipc", "warn: [{}] {} emit failed: {}", MethodNameOwned, Channel, Error);
							return Err(format!("[{}] emit failed: {}", MethodNameOwned, Error));
						}

						// Block until the frontend calls ResolveUIRequest with
						// the same nonce, or the sender is dropped (dialog
						// dismissed / window closed).
						match rx.await {
							Ok(Ok(Value)) => Ok(Value),
							Ok(Err(CommonError)) => Err(CommonError.to_string()),
							Err(_RecvError) => {
								// Sender was dropped without a reply -- the user
								// dismissed the dialog.  Return null so the extension
								// host sees `undefined` (VS Code contract for cancelled
								// quick-pick / input-box).
								dev_log!("ipc", "[{}] dialog dismissed (nonce dropped)", MethodNameOwned);
								Ok(Value::Null)
							},
						}
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
