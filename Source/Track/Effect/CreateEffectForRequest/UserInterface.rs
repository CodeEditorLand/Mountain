
//! # UserInterface Effect (CreateEffectForRequest)
//!
//! Effect constructors for user-interface dialog methods. Delegates to the
//! `UserInterfaceProvider` trait on `MountainEnvironment` for all UI
//! interactions.
//!
//! ## Methods handled
//!
//! | Method | Description |
//! |---|---|
//! | `UserInterface.ShowMessage` | Show a notification with optional severity |
//! | `UserInterface.ShowQuickPick` | Display a quick-pick selection list |
//! | `UserInterface.ShowInputBox` | Display a text input dialog |
//! | `UserInterface.ShowOpenDialog` | Display a file/folder picker dialog |
//! | `UserInterface.ShowSaveDialog` | Display a save file dialog |

use std::{future::Future, pin::Pin, sync::Arc};

use CommonLibrary::{
	Environment::Requires::Requires,
	UserInterface::{DTO::MessageSeverity::MessageSeverity, UserInterfaceProvider::UserInterfaceProvider},
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, Track::Effect::MappedEffectType::MappedEffect, dev_log};

pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"UserInterface.ShowMessage" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
						let severity_str = Parameters.get(0).and_then(Value::as_str).unwrap_or("info");
						let message = Parameters.get(1).and_then(Value::as_str).unwrap_or("").to_string();
						let options = Parameters.get(2).cloned();
						let severity = match severity_str {
							"warning" => MessageSeverity::Warning,
							"error" => MessageSeverity::Error,
							_ => MessageSeverity::Info,
						};
						provider
							.ShowMessage(severity, message, options)
							.await
							.map(|_| json!(null))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"UserInterface.ShowQuickPick" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
						let items = Parameters
							.get(0)
							.and_then(Value::as_array)
							.cloned()
							.unwrap_or_default()
							.into_iter()
							.filter_map(|v| {
								serde_json::from_value::<
									CommonLibrary::UserInterface::DTO::QuickPickItemDTO::QuickPickItemDTO,
								>(v)
								.ok()
							})
							.collect::<Vec<_>>();
						let options = Parameters.get(1).and_then(|V| {
							if V.is_object() {
								match serde_json::from_value::<
									CommonLibrary::UserInterface::DTO::QuickPickOptionsDTO::QuickPickOptionsDTO,
								>(V.clone())
								{
									Ok(dto) => Some(dto),
									Err(e) => {
										dev_log!("ipc", "warn: Failed to deserialize QuickPickOptionsDTO: {}", e);
										Some(Default::default())
									},
								}
							} else {
								None
							}
						});
						provider
							.ShowQuickPick(items, options)
							.await
							.map(|selected_items| json!(selected_items))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"UserInterface.ShowInputBox" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
						let options = if let Some(Value::Object(obj)) = Parameters.get(0) {
							match serde_json::from_value::<
								CommonLibrary::UserInterface::DTO::InputBoxOptionsDTO::InputBoxOptionsDTO,
							>(Value::Object(obj.clone()))
							{
								Ok(dto) => Some(dto),
								Err(e) => {
									dev_log!("ipc", "warn: Failed to deserialize InputBoxOptionsDTO: {}", e);
									Some(
										CommonLibrary::UserInterface::DTO::InputBoxOptionsDTO::InputBoxOptionsDTO::default(),
									)
								},
							}
						} else {
							None
						};
						provider
							.ShowInputBox(options)
							.await
							.map(|input_opt| json!(input_opt))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"UserInterface.ShowOpenDialog" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
						let options = if let Some(Value::Object(obj)) = Parameters.get(0) {
							match serde_json::from_value::<
								CommonLibrary::UserInterface::DTO::OpenDialogOptionsDTO::OpenDialogOptionsDTO,
							>(Value::Object(obj.clone()))
							{
								Ok(dto) => Some(dto),
								Err(e) => {
									dev_log!("ipc", "warn: Failed to deserialize OpenDialogOptionsDTO: {}", e);
									Some(Default::default())
								},
							}
						} else {
							None
						};
						provider
							.ShowOpenDialog(options)
							.await
							.map(|path_buf_opt| json!(path_buf_opt))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		"UserInterface.ShowSaveDialog" => {
			let effect =
				move |run_time:Arc<ApplicationRunTime>| -> Pin<Box<dyn Future<Output = Result<Value, String>> + Send>> {
					Box::pin(async move {
						let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();
						let options = if let Some(Value::Object(obj)) = Parameters.get(0) {
							match serde_json::from_value::<
								CommonLibrary::UserInterface::DTO::SaveDialogOptionsDTO::SaveDialogOptionsDTO,
							>(Value::Object(obj.clone()))
							{
								Ok(dto) => Some(dto),
								Err(e) => {
									dev_log!("ipc", "warn: Failed to deserialize SaveDialogOptionsDTO: {}", e);
									Some(Default::default())
								},
							}
						} else {
							None
						};
						provider
							.ShowSaveDialog(options)
							.await
							.map(|path_buf_opt| json!(path_buf_opt))
							.map_err(|e| e.to_string())
					})
				};

			Some(Ok(Box::new(effect)))
		},

		_ => None,
	}
}
