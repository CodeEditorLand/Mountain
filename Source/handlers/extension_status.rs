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
//   activation details. Emits a Tauri event `mountain://extension/activated`.
// - Handling `$onExtensionActivationError` notification: Logs the error
//   details. Emits `mountain://extension/activation_error`.
// - Handling `$onExtensionRuntimeError` notification: Logs the error details.
//   Emits `mountain://extension/runtime_error`.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` for specific notification
//   methods.
// - Primarily uses `log` for recording extension lifecycle events.
// - Emits Tauri events via `AppHandle::emit` to inform other parts of Mountain
//   or the Sky frontend about extension status changes.
// - Does not typically return data back to Cocoon for these notifications.
// --------------------------------------------------------------------------------------------

use log::{error, info, trace, warn};
use serde_json::Value;
// Manager is needed for app_handle.emit
use tauri::{AppHandle, Emitter, Runtime};

// For consistent error formatting if this handler itself encounters an issue
// (e.g., params not being an array).
use crate::handlers::error_utils;

/// Generic handler for various extension host status notifications received
/// from Cocoon.
///
/// This function processes notifications like extension activation,
///
///
/// deactivation, and errors. It logs these events and may emit Tauri events for
/// other parts of Mountain or the Sky frontend to react to.
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`, used for emitting events.
/// * `method` - The specific notification method name (e.g.,
///
///
///   `$onDidActivateExtension`).
/// * `params_val` - The `serde_json::Value` containing parameters from Cocoon.
///   This is expected by `track.rs` to be a `Value::Array`.
///
/// # Returns
/// * `Ok(Value::Null)` as notifications typically don't require a response.
/// * `Err(String)` if the `params_val` is not in the expected array format.
pub async fn handle_extension_host_status_notification<R:Runtime>(
	app_handle:AppHandle<R>,

	method:&str,

	params_val:Value,
) -> Result<Value, String> {
	// Notifications are fire-and-forget, so Result<Value, String> is mainly for
	// consistency with the Track dispatcher's signature and to handle internal
	// errors like param parsing.
	let params_array = match params_val.as_array() {
		Some(arr) => arr,

		None => {
			let err_msg = format!(
				"Parameters for notification method '{}' should be an array, but received type: {:?}. This is an \
				 internal error in how notifications are dispatched or received.",
				method,
				params_val.kind()
			);

			error!("[ExtStatus Handler] {}", err_msg);

			// This error is for the Track dispatcher if params_val isn't an array as
			// expected. It indicates a problem with the message structure from Cocoon or
			// Vine.
			return Err(error_utils::rpc_param_error_string(
				// Expected type
				method, "params", "array", // No specific index, the whole value is wrong
				None,
			));
		},
	};

	// Extract extension ID if available. It's typically the first parameter for
	// these notifications and is an `ExtensionIdentifier` DTO `{ value:
	// "pub.name", uuid?: "..." }`.
	let extension_id_dto = params_array.get(0);

	let extension_id_str = extension_id_dto
		 // Get the 'value' field from the DTO
		.and_then(|v| v.get("value"))
		.and_then(Value::as_str)
		 // Convert to owned String
		.map(String::from)
		.unwrap_or_else(|| {

			warn!(
				"[ExtStatus Handler] Could not parse string extension ID from DTO: {:?}. Using 'unknown_extension'.",


				extension_id_dto
			);


			"unknown_extension".to_string()
		});

	trace!(
		"[ExtStatus Handler] Method: {}, ExtID: '{}', Params Count: {}",
		method,
		extension_id_str,
		params_array.len()
	);

	match method {
		"$onWillActivateExtension" => {
			// Params: [extensionIdDto: IExtensionIdentifierDto]
			info!(
				"[ExtStatus Handler] <= {}: Extension '{}' attempting activation.",
				method, extension_id_str
			);

			// TODO: Future: Update some internal state in AppState, e.g.,

			// AppState.activating_extensions.insert(extension_id_str.clone());

			// This could be useful for tracking activation progress or
			// timeouts.
		},

		"$onDidActivateExtension" => {
			// Params: [extensionIdDto, startup: boolean, codeLoadingTime: number,

			// activateCallTime: number, activateResolvedTime: number, activationReason:
			// IActivationReasonDto]
			let startup = params_array.get(1).and_then(Value::as_bool).unwrap_or(false);

			// Use -1.0 or Option<f64> to indicate missing/invalid time values.
			let code_loading_time = params_array.get(2).and_then(Value::as_f64).unwrap_or(-1.0);

			let activate_call_time = params_array.get(3).and_then(Value::as_f64).unwrap_or(-1.0);

			let activate_resolved_time = params_array.get(4).and_then(Value::as_f64).unwrap_or(-1.0);

			let activation_reason_dto = params_array.get(5);

			// VS Code IActivationReason structure: { startup: boolean, extensionId?:
			// IExtensionIdentifier, activationEvent?: string }

			let activation_event_str = activation_reason_dto
				.and_then(|r_dto| r_dto.get("activationEvent"))
				.and_then(Value::as_str)
				 // Not Applicable or Not Available
				.unwrap_or("N/A");

			info!(
				"[ExtStatus Handler] <= {}: Extension '{}' ACTIVATED. Startup: {}, LoadTime: {:.2}ms, CallTime: \
				 {:.2}ms, ResolveTime: {:.2}ms, Event: '{}'",
				method,
				extension_id_str,
				startup,
				code_loading_time,
				activate_call_time,
				activate_resolved_time,
				activation_event_str
			);

			// TODO: Future: Update internal state, e.g.,

			// AppState.active_extensions.insert(extension_id_str.clone());

			// AppState.activating_extensions.remove(&extension_id_str);

			// Emit a Tauri event for other parts of Mountain or Sky to know.
			if let Err(e) =
				app_handle.emit("mountain://extension/activated", serde_json::json!({ "id": extension_id_str }))
			{
				warn!(
					"[ExtStatus Handler] Failed to emit mountain://extension/activated for {}: {}",
					extension_id_str, e
				);
			}
		},

		"$onExtensionActivationError" => {
			// Params: [extensionIdDto, error: SerializedError | string]
			// SerializedError: { name?: string, message?: string, stack?: string }

			let error_details_val = params_array.get(1);

			let error_message = error_details_val
				.and_then(|e_val| {
					if e_val.is_string() {
						e_val.as_str()
					} else {
						e_val.get("message").and_then(Value::as_str)
					}
				})
				.unwrap_or("Unknown activation error");

			error!(
				"[ExtStatus Handler] <= {}: Activation FAILED for extension '{}'. Message: '{}'. FullDetails: {:?}",
				method,
				extension_id_str,
				error_message,
				// Log full details if available
				error_details_val.unwrap_or(&Value::Null)
			);

			// TODO: Future: Update internal state, e.g.,

			// AppState.failed_extensions.insert(extension_id_str.clone(),

			// error_details_val.cloned().unwrap_or_default());

			// AppState.activating_extensions.remove(&extension_id_str);

			if let Err(e) = app_handle.emit(
				"mountain://extension/activation_error",
				serde_json::json!({

					"id": extension_id_str,


					 // Send full details to UI if possible
					"error": error_details_val.cloned().unwrap_or_default()
				}),
			) {
				warn!(
					"[ExtStatus Handler] Failed to emit mountain://extension/activation_error for {}: {}",
					extension_id_str, e
				);
			}
		},

		"$onExtensionRuntimeError" => {
			// Params: [extensionIdDto, error: SerializedError]
			// SerializedError usually has message, name, stack.
			let error_details_val = params_array.get(1);

			let error_message = error_details_val
				.and_then(|e_val| e_val.get("message"))
				.and_then(Value::as_str)
				.unwrap_or("Unknown runtime error");

			error!(
				"[ExtStatus Handler] <= {}: Runtime ERROR in extension '{}'. Message: '{}'. FullDetails: {:?}",
				method,
				extension_id_str,
				error_message,
				error_details_val.unwrap_or(&Value::Null)
			);

			// TODO: Future: Increment error count for this extension in AppState.
			// Potentially disable the extension if it errors too frequently ("bad-actor"
			// detection).

			if let Err(e) = app_handle.emit(
				"mountain://extension/runtime_error",
				serde_json::json!({

					"id": extension_id_str,


					"error": error_details_val.cloned().unwrap_or_default()
				}),
			) {
				warn!(
					"[ExtStatus Handler] Failed to emit mountain://extension/runtime_error for {}: {}",
					extension_id_str, e
				);
			}
		},

		_ => {
			// This case should ideally not be reached if Track.rs routes correctly.
			warn!(
				"[ExtStatus Handler] Received unknown or unhandled extension status notification method: '{}' with \
				 params: {:?}",
				method, params_array
			);
		},
	}

	// Notifications typically don't require a specific return value to the caller
	// (Vine/Track).
	Ok(Value::Null)
}
