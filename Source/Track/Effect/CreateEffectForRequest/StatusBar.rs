
//! # StatusBar Effect (CreateEffectForRequest)
//!
//! Effect constructors for status bar RPC methods from the Cocoon extension
//! host. Delegates to the `StatusBarProvider` trait on `MountainEnvironment`.
//!
//! ## Methods handled
//!
//! | Method | Description |
//! |---|---|
//! | `$statusBar:set` | Create or update a status bar entry |
//! | `$statusBar:dispose` | Remove a status bar entry by ID |
//! | `$setStatusBarMessage` | Set a simple text message in the status bar |
//! | `$disposeStatusBarMessage` | Remove a status bar message by ID |

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	StatusBar::{DTO::StatusBarEntryDTO::StatusBarEntryDTO, StatusBarProvider::StatusBarProvider},
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"$statusBar:set" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();

						// The extension host serialises this as an object with named fields,
						// matching the shape used by the RPC layer (EntryIdentifier = id).
						let entry_id = Parameters
							.get("id")
							.and_then(Value::as_str)
							.ok_or_else(|| "$statusBar:set: missing 'id' field".to_string())?;

						let item_id = Parameters.get("itemId").and_then(Value::as_str).unwrap_or(entry_id);

						let ext_id = Parameters.get("extensionId").and_then(Value::as_str).unwrap_or("");

						let text = Parameters.get("text").and_then(Value::as_str).unwrap_or("").to_string();

						let tooltip = Parameters.get("tooltip").cloned();

						let command = Parameters.get("command").cloned();

						let color = Parameters.get("color").cloned();

						let background_color = Parameters.get("backgroundColor").cloned();

						let is_aligned_left = Parameters.get("alignLeft").and_then(Value::as_bool).unwrap_or(false);

						let priority = Parameters.get("priority").and_then(Value::as_f64);

						let accessibility = Parameters.get("accessibilityInformation").cloned();

						let entry = StatusBarEntryDTO {
							EntryIdentifier:entry_id.to_string(),
							ItemIdentifier:item_id.to_string(),
							ExtensionIdentifier:ext_id.to_string(),
							Name:None,
							Text:text,
							Tooltip:tooltip,
							HasTooltipProvider:false,
							Command:command,
							Color:color,
							BackgroundColor:background_color,
							IsAlignedLeft:is_aligned_left,
							Priority:priority,
							AccessibilityInformation:accessibility,
						};

						provider
							.SetStatusBarEntry(entry)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$statusBar:dispose" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();

						// Require a non-empty id - a missing id would silently target the
						// wrong entry (previously fell back to the literal string "id").
						let id = Parameters
							.get(0)
							.and_then(Value::as_str)
							.filter(|s| !s.is_empty())
							.ok_or_else(|| "$statusBar:dispose: missing or empty entry id".to_string())?;

						provider
							.DisposeStatusBarEntry(id.to_string())
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$setStatusBarMessage" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();

						let message_id = Parameters
							.get(0)
							.and_then(Value::as_str)
							.filter(|s| !s.is_empty())
							.ok_or_else(|| "$setStatusBarMessage: missing or empty message id".to_string())?;

						let text = Parameters.get(1).and_then(Value::as_str).unwrap_or("").to_string();

						provider
							.SetStatusBarMessage(message_id.to_string(), text)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"$disposeStatusBarMessage" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn StatusBarProvider> = run_time.Environment.Require();

						let message_id = Parameters
							.get(0)
							.and_then(Value::as_str)
							.filter(|s| !s.is_empty())
							.ok_or_else(|| "$disposeStatusBarMessage: missing or empty message id".to_string())?;

						provider
							.DisposeStatusBarMessage(message_id.to_string())
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
