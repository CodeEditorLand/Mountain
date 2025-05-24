// ---------------------------------------------------------------------------------------------
// Mountain Sky UI Response Handlers (handlers/sky_ui_responses.rs)
// --------------------------------------------------------------------------------------------
// Contains Tauri command handlers that the Sky frontend invokes to send back
// results from UI interactions (dialogs, quick picks, input boxes) initiated by
// Mountain's UiProvider effects. This module acts as the bridge for
// asynchronous UI operations where Mountain requests a UI action and Sky
// provides the outcome.
//
// Responsibilities:
// - Implementing the `sky_resolves_ui_request` Tauri command.
// - Receiving the `request_id`, `data_val` (on success/cancellation), and
//   `error_details_val` (on UI-side error) from Sky.
// - Retrieving the corresponding `oneshot::Sender` from
//   `AppState.pending_ui_requests` using the `request_id`.
// - Constructing a `Result<Value, CommonError>` based on the data received from
//   Sky.
// - Sending this result back to the waiting task in `environment.rs` (which
//   initiated the UI request) via the `oneshot::Sender`.
// - Handling cases where the request might have already timed out on Mountain's
//   side.
//
// Key Interactions:
// - Invoked by the Sky frontend via Tauri's `invoke` system.
// - Accesses `AppState.pending_ui_requests` (thread-safely) to find and consume
//   the `oneshot::Sender`.
// - Uses `tokio::sync::oneshot::Sender` to communicate results back to async
//   tasks in `environment.rs`.
// - Uses `Land_Common::errors::CommonError` for error propagation.
// - Uses `handlers::error_utils` for formatting error strings returned by the
//   Tauri command itself (if this handler encounters an internal issue).
// --------------------------------------------------------------------------------------------

use Land_Common::errors::CommonError;
use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};

// For formatting errors returned by this command itself
use crate::{app_state::AppState, handlers::error_utils};

/// Formats a lock error specifically for the context of this Tauri command
/// failing. This error is for the `Result<(), String>` of the Tauri command
/// itself, not for the `oneshot::Sender`.
fn format_lock_error_for_command<T>(
	e:std::sync::PoisonError<std::sync::MutexGuard<'_, T>>,

	// e.g., "pending UI requests"
	context:&str,
) -> String {
	let user_facing_message = format!(
		"Internal server error while processing UI response: Could not access shared '{}' state. Please try the UI \
		 action again or restart the application if the issue persists.",
		context
	);

	let internal_log_message = format!(
		"[Sky UI Resp LockErr] CRITICAL: Failed to acquire lock on {} state: {}",
		context, e
	);

	error!("{}", internal_log_message);

	error_utils::rpc_error_string(user_facing_message, Some("ELOCKED_UI_RESPONSE_HANDLER"))
}

/// Helper to send the result (or error) via the oneshot sender and log.
/// This communicates the outcome of the UI operation back to the waiting task
/// in `environment.rs`.
fn send_ui_result_to_environment(
	request_id:&str,

	sender:tokio::sync::oneshot::Sender<Result<Value, CommonError>>,

	result_to_send:Result<Value, CommonError>,
) {
	let outcome_log = match &result_to_send {
		Ok(v) if v.is_null() => "cancellation/no-data",

		Ok(_) => "success data",

		Err(e) => {
			// Log the CommonError that will be sent back
			error!(
				"[Sky UI Resp] Preparing to send CommonError for ReqID '{}' to environment: {:?}",
				request_id, e
			);

			"error"
		},
	};

	if sender.send(result_to_send).is_err() {
		// This means the receiving end of the oneshot channel (in environment.rs) was
		// dropped. This usually happens if the UiProvider method in environment.rs
		// timed out before Sky could call back with this response.
		warn!(
			"[Sky UI Resp] For ReqID '{}': Failed to send {} result back to Mountain's waiting task. Receiver dropped \
			 (likely UiProvider method timed out or task cancelled by Mountain). Sky's response was effectively too \
			 late or the original request context is gone.",
			request_id, outcome_log
		);
	} else {
		info!(
			"[Sky UI Resp] For ReqID '{}': Successfully relayed Sky's {} response to Mountain's waiting task in \
			 environment.rs.",
			request_id, outcome_log
		);
	}
}

/// Generic Tauri command handler for Sky to send back results of any UI
/// interaction initiated by Mountain's UiProvider effects.
///
/// Sky should invoke this command with:
/// - `request_id`: The ID originally sent by Mountain with the UI request
///   event.
/// - `data_val`: `Option<Value>`.
///   - For successful interactions returning data (e.g., selected file paths,
///
///     input string), this should be the data serialized as a
///     `serde_json::Value`.
///   - For cancellations or interactions that don't return data but succeeded
///     (e.g., a simple message box closed), Sky should send `None` or
///     `Value::Null` for this field. `None` is preferred for clarity.
/// - `error_details_val`: `Option<Value>`.
///   - If an error occurred within Sky while processing/displaying the UI, or
///     if the user's action in Sky resulted in an error state defined by Sky,
///
///     Sky should send error details here (e.g., `json!({"message": "UI
///     component failed", "code": "ESKY_ERROR"})`).
///   - If the UI interaction was successful or normally cancelled by the user
///     without error, this should be `None`.
#[tauri::command]
pub async fn sky_resolves_ui_request(
	// Automatically injected by Tauri
	app_handle:AppHandle,

	request_id:String,

	data_val:Option<Value>,

	error_details_val:Option<Value>,
) -> Result<(), String> {
	// This Result<(), String> is for the Tauri command's own execution status
	info!(
		"[Sky UI Resp] Received UI response for ReqID='{}': DataIsSome={}, ErrorIsSome={}",
		request_id,
		data_val.is_some(),
		error_details_val.is_some()
	);

	trace!(
		"[Sky UI Resp] ReqID='{}': Data='{:?}', Error='{:?}'",
		request_id, data_val, error_details_val
	);

	let app_state = app_handle.state::<AppState>();

	let maybe_sender = {
		// Scope the lock
		let mut pending_guard = app_state
			.pending_ui_requests
			.lock()
			.map_err(|e| format_lock_error_for_command(e, "pending UI requests map"))?;

		// Check if the request is still pending before removing.
		// This helps avoid warnings if Sky calls back after Mountain has already timed
		// out and cleaned up.
		if !pending_guard.contains_key(&request_id) {
			warn!(
				"[Sky UI Resp] No pending UI request found for ReqID '{}' upon checking map (already handled, timed \
				 out, or invalid ID). Sky's response will be ignored.",
				request_id
			);

			// Command itself processed fine, but no further action needed.
			return Ok(());
		}
		// Remove and get the sender
		pending_guard.remove(&request_id)
	};

	if let Some(sender) = maybe_sender {
		let result_to_send:Result<Value, CommonError> = match (data_val, error_details_val) {
			// Case 1: Error reported from Sky. This takes precedence over any data.
			(_, Some(err_val)) => {
				let err_msg_str = err_val
					.get("message")
					.and_then(Value::as_str)
					.unwrap_or_else(|| err_val.as_str().unwrap_or("Unknown error structure reported by Sky UI"))
					.to_string();

				// Sky might send a code
				let err_code_str = err_val.get("code").and_then(Value::as_str);

				warn!(
					"[Sky UI Resp] ReqID '{}' resolved with an error from Sky: msg='{}', code='{:?}'",
					request_id,
					err_msg_str,
					err_code_str.unwrap_or("N/A")
				);

				// Package this as a CommonError::UiInteraction to send back to the waiting
				// effect
				Err(CommonError::UiInteraction(format!(
					"Error from Sky UI (ReqID: {}): {} (Code: {})",
					request_id,
					err_msg_str,
					// Provide a default code if Sky doesn't
					err_code_str.unwrap_or("SKY_UI_ERROR")
				)))
			},

			// Case 2: Success with data from Sky
			(Some(data), None) => {
				debug!("[Sky UI Resp] ReqID '{}' resolved successfully with data from Sky.", request_id);

				trace!("[Sky UI Resp] ReqID '{}' data: {:?}", request_id, data);

				Ok(data)
			},

			// Case 3: Success with no specific data (e.g., user cancellation, simple ack), or data was explicitly null
			(None, None) => {
				debug!(
					"[Sky UI Resp] ReqID '{}' resolved by Sky with no data and no error (e.g., user cancellation or \
					 void success).",
					request_id
				);

				// Represent cancellation or no data with Value::Null
				Ok(Value::Null)
			},
		};

		send_ui_result_to_environment(&request_id, sender, result_to_send);
	} else {
		// This case means the sender was not found after the lock was released,

		// which should be rare if the contains_key check passed, but good for
		// robustness.
		warn!(
			"[Sky UI Resp] No pending UI request sender found for ReqID '{}' after lock (unexpected state or race). \
			 Sky's response ignored.",
			request_id
		);
	}
	// The Tauri command itself (sky_resolves_ui_request) completed its processing
	// successfully.
	Ok(())
}

// Specific handlers like sky_resolves_open_dialog are no longer strictly
// necessary if sky_resolves_ui_request is used generically and Sky sends
// appropriate `data_val` structures that the corresponding UiProvider methods
// in environment.rs can parse from Value.
