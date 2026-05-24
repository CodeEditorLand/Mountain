//! Window-namespace UI commands from Cocoon's window shim.
//! ShowMessage is fire-and-forget (no selection reply needed).
//! ShowQuickPick / ShowInputBox / ShowOpenDialog / ShowSaveDialog block on
//! a oneshot channel that is resolved by the frontend via ResolveUIRequest.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	ApplicationState::Struct::ApplicationState::ApplicationState,
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::{
		CreateEffectForRequest::Utilities::Params::{ArrayUnwrap, EnsureArray},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

pub fn Fn<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"Window.ShowMessage" => {
			crate::effect!(RunTime, {
				use std::sync::atomic::{AtomicU64, Ordering as AO};

				use tauri::Emitter;

				let AppHandle = RunTime.Environment.ApplicationHandle.clone();
				let Payload = ArrayUnwrap(Parameters);
				let Message = Payload.get("message").and_then(Value::as_str).unwrap_or("").to_string();
				let Level = Payload.get("level").and_then(Value::as_str).unwrap_or("info").to_string();
				let Items = Payload.get("items").and_then(Value::as_array).cloned().unwrap_or_default();
				let Options = Payload.get("options").cloned().unwrap_or(json!({}));

				if Items.is_empty() {
					// Fire-and-forget: no action buttons needed.
					let _ = AppHandle.emit(
						"sky://notification/show",
						json!({
							"message": Message,
							"severity": Level,
							"actions": [],
							"options": Options,
						}),
					);
					return Ok(Value::Null);
				}

				// Round-trip: emit to the show-message-request channel
				// (which INotificationService handles with real action
				// buttons) and block until the user clicks or dismisses.
				static UI_MSG_SEQ:AtomicU64 = AtomicU64::new(1);
				let Nonce = format!("msg-{}", UI_MSG_SEQ.fetch_add(1, AO::Relaxed));

				let (tx, rx) = tokio::sync::oneshot::channel();
				RunTime.Environment.ApplicationState.UI.AddPendingRequest(Nonce.clone(), tx);

				let Actions:Vec<serde_json::Value> = Items
					.iter()
					.map(|Item| if Item.is_string() { json!({ "title": Item }) } else { Item.clone() })
					.collect();

				if let Err(Error) = AppHandle.emit(
					"sky://ui/show-message-request",
					json!({
						"RequestIdentifier": Nonce,
						"Payload": {
							"Severity": Level,
							"Message": Message,
							"Options": { "Actions": Actions },
						},
					}),
				) {
					RunTime.Environment.ApplicationState.UI.RemovePendingRequest(&Nonce);
					dev_log!("notification", "warn: [Window.ShowMessage] emit failed: {}", Error);
					return Ok(Value::Null);
				}

				match rx.await {
					Ok(Ok(Value)) => Ok(Value),
					_ => Ok(Value::Null),
				}
			})
		},

		"Window.ShowQuickPick" | "Window.ShowInputBox" | "Window.ShowOpenDialog" | "Window.ShowSaveDialog" => {
			let MethodNameOwned = MethodName.to_string();

			crate::effect!(RunTime, {
				use tauri::Emitter;

				let Args = EnsureArray(Parameters);

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
				RunTime.Environment.ApplicationState.UI.AddPendingRequest(Nonce.clone(), tx);

				let AppHandle = RunTime.Environment.ApplicationHandle.clone();
				if let Err(Error) = AppHandle.emit(Channel, json!({ "nonce": Nonce, "args": Args })) {
					// Emit failed -- remove the dangling sender so the map
					// does not grow unboundedly on repeated failures.
					RunTime.Environment.ApplicationState.UI.RemovePendingRequest(&Nonce.clone());
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
		},

		_ => None,
	}
}
