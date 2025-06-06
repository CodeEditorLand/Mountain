// File: Handlers/ExtensionStatus/ExtensionStatus.rs
// Defines the handler for processing extension lifecycle status notifications
// received from the Cocoon sidecar.

#![allow(non_snake_case, non_camel_case_types)]

use log::{error, info, trace, warn};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Runtime};

use crate::Handlers::ErrorUtils;

/// Processes notifications about the extension host's status, such as extension
/// activation events.
pub async fn HandleExtensionHostStatusNotification<R:Runtime>(
	ApplicationHandle:AppHandle<R>,
	MethodName:&str,
	ParametersValue:Value,
) -> Result<Value, String> {
	let ParametersArray = match ParametersValue.as_array() {
		Some(Array) => Array,
		None => {
			let ErrorMessage = format!(
				"Parameters for notification method '{}' should be an array, but received type: {:?}.",
				MethodName,
				ParametersValue.kind()
			);
			error!("[ExtensionStatus Handler] {}", ErrorMessage);
			return Err(ErrorUtils::RpcParamErrorString(MethodName, "params", "array", None));
		},
	};

	let ExtensionIdentifierDto = ParametersArray.get(0);
	let ExtensionIdentifierString = ExtensionIdentifierDto
		.and_then(|ValueItem| ValueItem.get("value"))
		.and_then(Value::as_str)
		.map(String::from)
		.unwrap_or_else(|| {
			warn!(
				"[ExtensionStatus Handler] Could not parse string extension ID from DTO: {:?}. Using \
				 'unknown_extension'.",
				ExtensionIdentifierDto
			);
			"unknown_extension".to_string()
		});

	trace!(
		"[ExtensionStatus Handler] Method: {}, ExtensionIdentifier: '{}', Parameters Count: {}",
		MethodName,
		ExtensionIdentifierString,
		ParametersArray.len()
	);

	match MethodName {
		"$onWillActivateExtension" => {
			info!(
				"[ExtensionStatus Handler] {}: Extension '{}' attempting activation.",
				MethodName, ExtensionIdentifierString
			);
			// Future logic could involve updating a UI element to show
			// "Activating..."
		},
		"$onDidActivateExtension" => {
			let IsStartup = ParametersArray.get(1).and_then(Value::as_bool).unwrap_or(false);
			let CodeLoadingTime = ParametersArray.get(2).and_then(Value::as_f64).unwrap_or(-1.0);
			let ActivateCallTime = ParametersArray.get(3).and_then(Value::as_f64).unwrap_or(-1.0);
			let ActivateResolvedTime = ParametersArray.get(4).and_then(Value::as_f64).unwrap_or(-1.0);
			let ActivationReasonDto = ParametersArray.get(5);
			let ActivationEventString = ActivationReasonDto
				.and_then(|Dto| Dto.get("activationEvent"))
				.and_then(Value::as_str)
				.unwrap_or("N/A");

			info!(
				"[ExtensionStatus Handler] {}: Extension '{}' ACTIVATED. Startup: {}, LoadTime: {:.2}ms, CallTime: \
				 {:.2}ms, ResolveTime: {:.2}ms, Event: '{}'",
				MethodName,
				ExtensionIdentifierString,
				IsStartup,
				CodeLoadingTime,
				ActivateCallTime,
				ActivateResolvedTime,
				ActivationEventString
			);

			if let Err(Error) = ApplicationHandle.emit(
				"mountain:extension_activated",
				serde_json::json!({
					"id": ExtensionIdentifierString
				}),
			) {
				warn!(
					"[ExtensionStatus Handler] Failed to emit mountain:extension_activated for '{}': {}",
					ExtensionIdentifierString, Error
				);
			}
		},
		"$onExtensionActivationError" => {
			let ErrorDetailsValue = ParametersArray.get(1);
			let ErrorMessage = ErrorDetailsValue
				.and_then(|ValueItem| {
					if ValueItem.is_string() {
						ValueItem.as_str()
					} else {
						ValueItem.get("message").and_then(Value::as_str)
					}
				})
				.unwrap_or("Unknown activation error");

			error!(
				"[ExtensionStatus Handler] {}: Activation FAILED for extension '{}'. Message: '{}'. FullDetails: {:?}",
				MethodName,
				ExtensionIdentifierString,
				ErrorMessage,
				ErrorDetailsValue.unwrap_or(&Value::Null)
			);

			if let Err(Error) = ApplicationHandle.emit(
				"mountain:extension_activation_failed",
				serde_json::json!({
					"id": ExtensionIdentifierString,
					"error": ErrorDetailsValue.cloned().unwrap_or_default()
				}),
			) {
				warn!(
					"[ExtensionStatus Handler] Failed to emit mountain:extension_activation_failed for '{}': {}",
					ExtensionIdentifierString, Error
				);
			}
		},
		"$onExtensionRuntimeError" => {
			let ErrorDetailsValue = ParametersArray.get(1);
			let ErrorMessage = ErrorDetailsValue
				.and_then(|ValueItem| ValueItem.get("message"))
				.and_then(Value::as_str)
				.unwrap_or("Unknown runtime error");

			error!(
				"[ExtensionStatus Handler] {}: Runtime ERROR in extension '{}'. Message: '{}'. FullDetails: {:?}",
				MethodName,
				ExtensionIdentifierString,
				ErrorMessage,
				ErrorDetailsValue.unwrap_or(&Value::Null)
			);

			if let Err(Error) = ApplicationHandle.emit(
				"mountain:extension_runtime_error",
				serde_json::json!({
					"id": ExtensionIdentifierString,
					"error": ErrorDetailsValue.cloned().unwrap_or_default()
				}),
			) {
				warn!(
					"[ExtensionStatus Handler] Failed to emit mountain:extension_runtime_error for '{}': {}",
					ExtensionIdentifierString, Error
				);
			}
		},
		_ => {
			warn!(
				"[ExtensionStatus Handler] Received unknown extension status notification method: '{}' with params: \
				 {:?}",
				MethodName, ParametersArray
			);
		},
	}
	Ok(Value::Null)
}
