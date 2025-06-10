use log::{error, info, trace, warn};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, Runtime};

/// @module StatusNotificationLogic
/// @description Contains the logic for processing extension lifecycle status
/// notifications sent from the Cocoon sidecar back to the Mountain host.
use crate::handlers::error_utils;

/// Processes notifications about the extension host's status, such as when an
/// extension successfully activates or fails to activate. This function is
/// typically called by the `vine` gRPC server when it receives a notification
/// from Cocoon.
///
/// @param AppHandle - The Tauri application handle.
/// @param MethodName - The name of the notification method (e.g.,
/// `$onDidActivateExtension`). @param ParametersValue - The parameters sent
/// with the notification.
pub async fn HandleExtensionHostStatusNotification<R:Runtime>(
	AppHandle:&AppHandle<R>,
	MethodName:&str,
	ParametersValue:Value,
) -> Result<Value, String> {
	let ParametersArray = ParametersValue
		.as_array()
		.ok_or_else(|| error_utils::RpcParamErrorString(MethodName, "params", "array", None))?;

	// The first parameter is usually the extension identifier DTO.
	let ExtensionIdString = ParametersArray
		.get(0)
		.and_then(|v| v.get("value"))
		.and_then(Value::as_str)
		.unwrap_or("unknown")
		.to_string();

	trace!(
		"[ExtensionStatus] Received notification: {}, ExtID: '{}'",
		MethodName, ExtensionIdString
	);

	match MethodName {
		"$onDidActivateExtension" => {
			info!("[ExtensionStatus] Extension ACTIVATED: '{}'", ExtensionIdString);
			// Notify the frontend that the extension's state has changed.
			if let Err(e) = AppHandle.emit("sky://extension/activated", serde_json::json!({ "Id": ExtensionIdString }))
			{
				warn!("[ExtensionStatus] Failed to emit activated event to Sky: {}", e);
			}
		},
		"$onExtensionActivationError" => {
			let ErrorDetails = ParametersArray.get(1).cloned().unwrap_or_default();
			error!(
				"[ExtensionStatus] Activation FAILED for '{}': {:?}",
				ExtensionIdString, ErrorDetails
			);
			// Notify the frontend of the error so it can display a notification.
			if let Err(e) = AppHandle.emit(
				"sky://extension/activation_failed",
				serde_json::json!({ "Id": ExtensionIdString, "Error": ErrorDetails }),
			) {
				warn!("[ExtensionStatus] Failed to emit activation_failed event to Sky: {}", e);
			}
		},
		_ => {
			trace!("[ExtensionStatus] Received unhandled status notification: {}", MethodName);
		},
	}

	// Notifications do not return a meaningful value.
	Ok(Value::Null)
}
