/// matches.
pub fn Matches(MethodName:&str) -> bool {
	MethodName.starts_with("UserInterface.") || MethodName.starts_with("Window.")
}

use std::sync::Arc;

use CommonLibrary::{
	Environment::Requires::Requires,
	UserInterface::{DTO::MessageSeverity::MessageSeverity, UserInterfaceProvider::UserInterfaceProvider},
};
use serde_json::{Value, json};
use tauri::Runtime;

use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	Track::Effect::{
		CreateEffectForRequest::Utilities::Params::{string_at, string_at_or},
		MappedEffectType::MappedEffect,
	},
	dev_log,
};

/// Creates effect.
pub fn CreateEffect<R:Runtime>(MethodName:&str, Parameters:Value) -> Option<Result<MappedEffect, String>> {
	match MethodName {
		"UserInterface.ShowMessage" => {
			crate::effect!(run_time, {
				let Parameters = Parameters.clone();

				let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();

				let severity_str = string_at_or(&Parameters, 0, "info");

				let message = string_at(&Parameters, 1);

				let options = Parameters.get(2).cloned();

				let severity = match severity_str.as_str() {
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
		},

		"UserInterface.ShowQuickPick" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();

				let items = Parameters
					.get(0)
					.and_then(Value::as_array)
					.cloned()
					.unwrap_or_default()
					.into_iter()
					.filter_map(|v| {
						serde_json::from_value::<CommonLibrary::UserInterface::DTO::QuickPickItemDTO::QuickPickItemDTO>(
							v,
						)
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
		},

		"UserInterface.ShowInputBox" => {
			crate::effect!(run_time, {
				let provider:Arc<dyn UserInterfaceProvider> = run_time.Environment.Require();

				let options = if let Some(Value::Object(obj)) = Parameters.get(0) {
					match serde_json::from_value::<
						CommonLibrary::UserInterface::DTO::InputBoxOptionsDTO::InputBoxOptionsDTO,
					>(Value::Object(obj.clone()))
					{
						Ok(dto) => Some(dto),
						Err(e) => {
							dev_log!("ipc", "warn: Failed to deserialize InputBoxOptionsDTO: {}", e);

							Some(CommonLibrary::UserInterface::DTO::InputBoxOptionsDTO::InputBoxOptionsDTO::default())
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
		},

		"UserInterface.ShowOpenDialog" => {
			crate::effect!(run_time, {
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
		},

		"UserInterface.ShowSaveDialog" => {
			crate::effect!(run_time, {
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
		},

		_ => None,
	}
}
