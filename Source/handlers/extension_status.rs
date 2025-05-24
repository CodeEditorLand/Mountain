// ---------------------------------------------------------------------------------------------
// Mountain Extension Host Status Handlers (handlers/extension_status.rs)
// --------------------------------------------------------------------------------------------
// Handles notifications from Cocoon related to the extension host's lifecycle
// and individual extension activation statuses. These are typically
// fire-and-forget notifications from Cocoon to Mountain, primarily for logging
// or potentially triggering other internal Mountain logic based on extension
// states.
//
// Responsibilities:
// - Handling `$onWillActivateExtension` notification: Logs the event.
// - Handling `$onDidActivateExtension` notification: Logs the event and
//   activation details.
// - Handling `$onExtensionActivationError` notification: Logs the error
//   details.
// - Handling `$onExtensionRuntimeError` notification: Logs the error details.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` for specific notification
//   methods.
// - Primarily uses `log` for recording extension lifecycle events.
// - Does not typically return data back to Cocoon for these notifications.
// --------------------------------------------------------------------------------------------

use log::{debug, error, info, trace, warn};
use serde_json::Value;
// Added Manager for potential future app_handle.emit_all
use tauri::{AppHandle, Manager, Runtime};

// For consistent error formatting if needed
use crate::handlers::error_utils;

/// Generic handler for various extension host status notifications from Cocoon.
/// `method`: The specific notification method name (e.g.,
///
/// "$onDidActivateExtension") `params_val`: The `Value` containing parameters
/// from Cocoon (Track.rs ensures this is often Value::Array).
pub async fn handle_ext_host_status<R:Runtime>(
	// Keep app_handle for potential future event emissions
	app_handle:AppHandle<R>,

	method:&str,

	params_val:Value,
) -> Result<Value, String> {
	// Still Result<Value, String> for consistency with Track dispatcher signature
	let params_array = match params_val.as_array() {
		Some(arr) => arr,

		None => {
			let err_msg = format!(
				"Parameters for notification method '{}' should be an array, but received type: {:?}",
				method,
				params_val.kind()
			);

			error!("[ExtStatus Handler] {}", err_msg);

			// This error is for the Track dispatcher if params_val isn't an array as
			// expected
			return Err(error_utils::rpc_param_error_string(method, "params", "array", None));
		},
	};

	// Extract extension ID if available (common first parameter for these
	// notifications)
	let extension_id_dto = params_array.get(0);

	let extension_id_str = extension_id_dto
         // Assuming DTO { value: "pub.name", uuid?: "..." }

		.and_then(|v| v.get("value"))
        .and_then(Value::as_str)
         // Fallback if parsing fails
		.unwrap_or("unknown_extension");

	trace!(
		"[ExtStatus] Method: {}, ExtID: {}, Params Count: {}",
		method,
		extension_id_str,
		params_array.len()
	);

	match method {
		"$onWillActivateExtension" => {
			info!(
				"[ExtStatus] <= {}: Extension '{}' attempting activation.",
				method, extension_id_str
			);

			// Future: Update some internal state, e.g.,

			// AppState.activating_extensions.insert(extension_id_str);
		},

		"$onDidActivateExtension" => {
			// Params: [extensionIdDto, startup: boolean, codeLoadingTime: number,

			// activateCallTime: number, activateResolvedTime: number, activationReason:
			// IActivationReason]
			let startup = params_array.get(1).and_then(Value::as_bool).unwrap_or(false);

			// Use -1 to indicate missing/invalid
			let code_loading_time = params_array.get(2).and_then(Value::as_f64).unwrap_or(-1.0);

			let activate_call_time = params_array.get(3).and_then(Value::as_f64).unwrap_or(-1.0);

			let activate_resolved_time = params_array.get(4).and_then(Value::as_f64).unwrap_or(-1.0);

			let activation_reason_dto = params_array.get(5);

			let activation_event_str = activation_reason_dto
                 // VS Code activationReason structure
				.and_then(|r| r.get("activationEvent"))
                .and_then(Value::as_str)
                .unwrap_or("N/A");

			info!(
				"[ExtStatus] <= {}: Extension '{}' ACTIVATED. Startup: {}, LoadTime: {:.2}ms, CallTime: {:.2}ms, \
				 ResolveTime: {:.2}ms, Event: '{}'",
				method,
				extension_id_str,
				startup,
				code_loading_time,
				activate_call_time,
				activate_resolved_time,
				activation_event_str
			);

			// Future: Update internal state, e.g.,

			// AppState.active_extensions.insert(extension_id_str); Example: Emit Tauri
			// event for other parts of Mountain or Sky to know
			if let Err(e) =
				app_handle.emit_all("mountain://extension/activated", serde_json::json!({ "id": extension_id_str }))
			{
				warn!(
					"[ExtStatus] Failed to emit mountain://extension/activated for {}: {}",
					extension_id_str, e
				);
			}
		},

		"$onExtensionActivationError" => {
			let error_details_val = params_array.get(1);

			let error_message = error_details_val
				.and_then(|e| e.get("message"))
				.and_then(Value::as_str)
				.unwrap_or("Unknown activation error");

			error!(
				"[ExtStatus] <= {}: Activation FAILED for extension '{}'. Message: '{}'. FullDetails: {:?}",
				method,
				extension_id_str,
				error_message,
				error_details_val.unwrap_or(&Value::Null)
			);

			// Future: Update internal state, e.g.,

			// AppState.failed_extensions.insert(extension_id_str, error_details);

			if let Err(e) = app_handle.emit_all(
				"mountain://extension/activation_error",
				serde_json::json!({
					"id": extension_id_str,

					 // Send details to UI
					"error": error_details_val.cloned().unwrap_or_default()
				}),
			) {
				warn!(
					"[ExtStatus] Failed to emit mountain://extension/activation_error for {}: {}",
					extension_id_str, e
				);
			}
		},

		"$onExtensionRuntimeError" => {
			// Params: [extensionIdDto, error: SerializedError] (SerializedError usually has
			// message, name, stack)
			let error_details_val = params_array.get(1);

			let error_message = error_details_val
				.and_then(|e| e.get("message"))
				.and_then(Value::as_str)
				.unwrap_or("Unknown runtime error");

			error!(
				"[ExtStatus] <= {}: Runtime ERROR in extension '{}'. Message: '{}'. FullDetails: {:?}",
				method,
				extension_id_str,
				error_message,
				error_details_val.unwrap_or(&Value::Null)
			);

			// Future: Increment error count for extension, potentially disable if too many
			// errors.
			if let Err(e) = app_handle.emit_all(
				"mountain://extension/runtime_error",
				serde_json::json!({
					"id": extension_id_str,

					"error": error_details_val.cloned().unwrap_or_default()
				}),
			) {
				warn!(
					"[ExtStatus] Failed to emit mountain://extension/runtime_error for {}: {}",
					extension_id_str, e
				);
			}
		},

		_ => {
			warn!(
				"[ExtStatus] Received unknown status notification method: '{}' with params: {:?}",
				method, params_array
			);
		},
	}

	// Notifications typically don't require a specific return value to the caller
	// (Vine/Track)
	Ok(Value::Null)
}
