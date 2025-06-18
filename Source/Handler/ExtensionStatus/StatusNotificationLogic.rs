// @module StatusNotificationLogic
// @description Contains the logic for processing extension lifecycle status
// notifications sent from the Cocoon sidecar back to the Mountain host.

use log::{error, info, trace, warn};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Runtime};

use crate::Handler::error_utils;

/// Processes notifications about the extension host's status, such as when an
/// extension successfully activates or fails to activate. This function is
/// typically called by the `Vine` gRPC server when it receives a notification
/// from Cocoon.
///
/// @param app_handle - The Tauri application handle.
/// @param method_name - The name of the notification method (e.g.,
/// `$onDidActivateExtension`). @param parameters_value - The parameters sent
/// with the notification.
pub async fn HandleExtensionHostStatusNotification<R:Runtime>(
	app_handle:&AppHandle<R>,
	method_name:&str,
	parameters_value:Value,
) -> Result<Value, String> {
	let parameters_array = parameters_value
		.as_array()
		.ok_or_else(|| error_utils::RpcParamErrorString(method_name, "params", "array", None))?;

	// The first parameter is usually the extension identifier DTO.
	let extension_id_string = parameters_array
		.get(0)
		.and_then(|v| v.get("value"))
		.and_then(Value::as_str)
		.unwrap_or("unknown")
		.to_string();

	trace!(
		"[ExtensionStatus] Received notification: {}, ExtID: '{}'",
		method_name, extension_id_string
	);

	match method_name {
		"$onDidActivateExtension" => {
			info!("[ExtensionStatus] Extension ACTIVATED: '{}'", extension_id_string);
			// Notify the frontend that the extension's state has changed.
			if let Err(e) =
				app_handle.emit("sky://extension/activated", serde_json::json!({ "Id": extension_id_string }))
			{
				warn!("[ExtensionStatus] Failed to emit activated event to Sky: {}", e);
			}
		},
		"$onExtensionActivationError" => {
			let error_details = parameters_array.get(1).cloned().unwrap_or_default();
			error!(
				"[ExtensionStatus] Activation FAILED for '{}': {:?}",
				extension_id_string, error_details
			);
			// Notify the frontend of the error so it can display a notification.
			if let Err(e) = app_handle.emit(
				"sky://extension/activation_failed",
				serde_json::json!({ "Id": extension_id_string, "Error": error_details }),
			) {
				warn!("[ExtensionStatus] Failed to emit activation_failed event to Sky: {}", e);
			}
		},
		_ => {
			trace!("[ExtensionStatus] Received unhandled status notification: {}", method_name);
		},
	}

	// Notifications do not return a meaningful value.
	Ok(Value::Null)
}
