// ---------------------------------------------------------------------------------------------
// Mountain Sky UI Response Handlers (handlers/sky_ui_responses.rs)
// --------------------------------------------------------------------------------------------
// Contains Tauri command handlers that the Sky frontend invokes to send back
// results from UI interactions (dialogs, quick picks, input boxes) initiated by
// Mountain's UiProvider effects (implemented in `environment.rs`). This module
// acts as the bridge for asynchronous UI operations where Mountain requests a
// UI action, Sky performs it, and then Sky calls back to Mountain with the
// outcome.
//
// Responsibilities:
// - Implementing the `sky_resolves_ui_request` Tauri command, which is the
//   single entry point for all UI responses from Sky.
// - Receiving the `request_id` (originally sent by Mountain), `data_val` (on
//   success or user cancellation), and `error_details_val` (if an error
//   occurred in Sky) from the frontend.
// - Retrieving the corresponding `tokio::sync::oneshot::Sender` from
//   `AppState.pending_ui_requests` using the `request_id`. This sender was
//   stored by the `UiProvider` method in `environment.rs` when it initiated the
//   UI request.
// - Constructing a `Result<Value, CommonError>` based on the data received from
//   Sky:
//   - If `error_details_val` is present, it's converted to a
//     `CommonError::UiInteraction`.
//   - If `data_val` is present, it's passed as `Ok(data_val)`.
//   - If both are `None` or `data_val` is `Value::Null`, it's treated as
//     `Ok(Value::Null)` (e.g., user cancellation).
// - Sending this `Result` back to the waiting asynchronous task in
//   `environment.rs` (which is awaiting the `oneshot::Receiver`) via the
//   retrieved `oneshot::Sender`.
// - Handling cases where the request might have already timed out on Mountain's
//   side (the `oneshot::Sender` would fail to send, which is logged).
//
// Key Interactions:
// - Invoked by the Sky frontend via Tauri's `invoke` system (e.g.,

//   `invoke('sky_resolves_ui_request', { ... })`).
// - Accesses `AppState.pending_ui_requests` (a thread-safe `HashMap`) to find
//   and consume the `oneshot::Sender` associated with a `request_id`.
// - Uses `tokio::sync::oneshot::Sender` to communicate results asynchronously
//   back to tasks in `environment.rs` that are awaiting UI outcomes.
// - Uses `Land_Common::errors::CommonError` for packaging errors to be sent
//   back to the `UiProvider` effect.
// - Uses `handlers::error_utils` for formatting error strings returned by the
//   `sky_resolves_ui_request` Tauri command itself if this handler encounters
//   an internal issue (e.g., lock poisoning).
// --------------------------------------------------------------------------------------------

use Land_Common::errors::CommonError;
use log::{debug, error, info, trace, warn};
// `json!` macro might not be used here, but `Value` is.
use serde_json::Value;
use tauri::{AppHandle, Manager};

// For formatting errors returned by this Tauri command itself (not the oneshot result)
use crate::{app_state::AppState, handlers::error_utils};

/// Formats a `PoisonError` from a Mutex lock specifically for the context of
/// this Tauri command failing to process a UI response.
///
/// This error string is intended for the `Result<(), String>` of the
/// `sky_resolves_ui_request` Tauri command itself, not for the `Result` sent
/// through the `oneshot::Sender` to the `UiProvider` effect.
///
/// # Arguments
/// * `e` - The `PoisonError` encountered.
/// * `context` - A string describing the locked resource (e.g., "pending UI
///   requests map").
///
/// # Returns
/// A `String` containing a JSON-formatted RPC error suitable for a Tauri
/// command's error response.
fn format_lock_error_for_tauri_command_failure<T>(
	e:std::sync::PoisonError<std::sync::MutexGuard<'_, T>>,

	context:&str,
) -> String {
	let user_facing_message = format!(
		"Internal server error while processing UI response: Could not access shared '{}' state. Please try the UI \
		 action again or restart the application if the issue persists.",
		context
	);

	let internal_log_message = format!(
		"[Sky UI Resp Handler LockErr] CRITICAL: Failed to acquire lock on {} state: {}. This prevents processing the \
		 UI response.",
		context, e
	);

	// Log detailed internal error
	error!("{}", internal_log_message);

	error_utils::rpc_error_string(
		user_facing_message,
		// Specific error code
		Some("ELOCKED_UI_RESPONSE_HANDLER"),
	)
}

/// Helper function to send the UI operation's result (or error) via the
/// `oneshot::Sender` back to the waiting task in `environment.rs`.
///
/// This function also logs the outcome of attempting to send the result.
///
/// # Arguments
/// * `request_id` - The unique ID of the UI request being resolved.
/// * `sender` - The `tokio::sync::oneshot::Sender` associated with the
///   `request_id`.
/// * `result_to_send` - The `Result<Value, CommonError>` to send, representing
///   the outcome of the UI operation.
fn send_ui_operation_result_to_environment_task(
	request_id:&str,

	sender:tokio::sync::oneshot::Sender<Result<Value, CommonError>>,

	result_to_send:Result<Value, CommonError>,
) {
	let outcome_log_description = match &result_to_send {
		Ok(v) if v.is_null() => "cancellation or no-data success",

		Ok(_) => "success with data",

		Err(e) => {
			// Log the CommonError that will be sent back to the environment task.
			error!(
				"[Sky UI Resp Handler] Preparing to send CommonError for ReqID '{}' back to environment task: {:?}",
				request_id, e
			);

			// Generic description for logging
			"error"
		},
	};

	if sender.send(result_to_send).is_err() {
		// This `is_err()` case means the `oneshot::Receiver` on the `environment.rs`
		// side was dropped. This typically happens if the `UiProvider` method in
		// `environment.rs` timed out waiting for Sky's response, or if the task
		// itself was cancelled for other reasons.
		warn!(
			"[Sky UI Resp Handler] For ReqID '{}': Failed to send {} result back to Mountain's waiting task in \
			 environment.rs. The receiver was dropped, likely because the original UiProvider method timed out or its \
			 task was cancelled. Sky's response for this request arrived too late or the originating context in \
			 Mountain is gone.",
			request_id, outcome_log_description
		);
	} else {
		info!(
			"[Sky UI Resp Handler] For ReqID '{}': Successfully relayed Sky's {} response to Mountain's waiting task \
			 in environment.rs.",
			request_id, outcome_log_description
		);
	}
}

/// Tauri command handler invoked by the Sky frontend to send back results of UI
/// interactions (dialogs, quick picks, input boxes) initiated by Mountain.
///
/// Sky should invoke this command with:
/// - `request_id`: The unique string ID that Mountain originally sent with the
///   UI request event (e.g., `sky://ui/show-open-dialog-request`).
/// - `data_val`: An `Option<Value>`.
///   - For successful interactions returning data (e.g., selected file paths
///     from an open dialog, input string from an input box), this should be
///     `Some(data_value)` where `data_value` is the `serde_json::Value`
///     representation of the data.
///   - For user cancellations (e.g., closing a dialog without selection) or
///     successful interactions that don't inherently return data (e.g., a
///     simple "OK" on a message box), Sky should send `None` or
///     `Some(Value::Null)`. `Some(Value::Null)` is often canonical for "no
///     result".
/// - `error_details_val`: An `Option<Value>`.
///   - If an error occurred *within Sky* while it was trying to process or
///     display the UI (e.g., a UI component failed, an internal Sky logic
///     error), Sky should send `Some(error_details_value)`. This
///     `error_details_value` should ideally be a JSON object like `{"message":
///     "Sky UI component failed", "code": "ESKY_ERROR_CODE"}`.
///   - If the UI interaction was successful or normally cancelled by the user
///     without any error on Sky's side, this should be `None`.
///
/// # Arguments
/// * `app_handle` - The Tauri `AppHandle`, automatically injected.
/// * `request_id` - The unique ID of the UI request being resolved.
/// * `data_val` - Optional `serde_json::Value` containing successful data or
///   null for cancellation.
/// * `error_details_val` - Optional `serde_json::Value` containing error
///   details if Sky encountered an error.
///
/// # Returns
/// * `Result<(), String>`:
///   - `Ok(())` if this handler successfully processed the response (i.e.,
///
///
///
///
///     found the pending request and attempted to send the result back to the
///     environment task).
///   - `Err(String)` containing a JSON-RPC error if this handler itself fails
///     (e.g., due to a poisoned lock when accessing `AppState`). This error is
///     for the Tauri command invocation, not the UI operation's outcome.
#[tauri::command]
pub async fn sky_resolves_ui_request(
	// Automatically injected by Tauri
	app_handle:AppHandle,

	request_id:String,

	// Data from Sky on success/cancellation
	data_val:Option<Value>,

	// Error details if Sky had an issue
	error_details_val:Option<Value>,
) -> Result<(), String> {
	// This `Result<(), String>` pertains to the execution status of this Tauri
	// command handler itself.
	info!(
		"[Sky UI Resp Handler] Received UI response for ReqID='{}': DataIsSome={}, ErrorIsSome={}",
		request_id,
		data_val.is_some(),
		error_details_val.is_some()
	);

	trace!(
		"[Sky UI Resp Handler] Full details for ReqID='{}': Data='{:?}', Error='{:?}'",
		request_id, data_val, error_details_val
	);

	let app_state = app_handle.state::<AppState>();

	let maybe_oneshot_sender = {
		// Scope the Mutex lock to minimize its duration.
		let mut pending_ui_requests_guard = app_state
			.pending_ui_requests
			.lock()
			.map_err(|e| format_lock_error_for_tauri_command_failure(e, "pending UI requests map"))?;

		// Check if the request is still pending before removing. This helps avoid
		// warnings if Sky calls back after Mountain has already timed out and cleaned
		// up.
		if !pending_ui_requests_guard.contains_key(&request_id) {
			warn!(
				"[Sky UI Resp Handler] No pending UI request found for ReqID '{}' in AppState.pending_ui_requests. \
				 The request might have already been handled, timed out and cleaned up by Mountain, or the ID is \
				 invalid. Sky's response will be ignored.",
				request_id
			);

			// Command itself processed fine (it found no pending request), but no further
			// action.
			return Ok(());
		}

		// Remove the sender from the map, consuming it.
		pending_ui_requests_guard.remove(&request_id)
	};

	if let Some(oneshot_sender) = maybe_oneshot_sender {
		// Determine the result to send back to the waiting task in `environment.rs`.
		let result_for_environment_task:Result<Value, CommonError> = match (data_val, error_details_val) {
			// Case 1: Error reported from Sky. This takes precedence over any data.
			(_, Some(sky_error_value)) => {
				let error_message_str = sky_error_value
					.get("message")
					.and_then(Value::as_str)
					.unwrap_or_else(|| {
						// Fallback if Sky's error format is unexpected
						sky_error_value
							.as_str()
							.unwrap_or("Unknown or malformed error structure reported by Sky UI")
					})
					.to_string();

				let sky_error_code_str = sky_error_value.get("code").and_then(Value::as_str);

				warn!(
					"[Sky UI Resp Handler] ReqID '{}' resolved with an error reported by Sky: msg='{}', code='{:?}'",
					request_id,
					error_message_str,
					sky_error_code_str.unwrap_or("N/A_SKY_CODE")
				);

				// Package this as a CommonError::UiInteraction to send back to the waiting
				// effect.
				Err(CommonError::UiInteraction(format!(
					"Error from Sky UI (ReqID: {}): {} (Sky Code: {})",
					request_id,
					error_message_str,
					sky_error_code_str.unwrap_or("SKY_UI_ERROR_NO_CODE") /* Provide a default if Sky doesn't send a
					                                                      * code */
				)))
			},

			// Case 2: Success with data from Sky.
			(Some(successful_data), None) => {
				debug!(
					"[Sky UI Resp Handler] ReqID '{}' resolved successfully with data from Sky.",
					request_id
				);

				trace!("[Sky UI Resp Handler] Data for ReqID '{}': {:?}", request_id, successful_data);

				Ok(successful_data)
			},

			// Case 3: Success with no specific data (e.g., user cancellation of a dialog,

			//         simple ack for a message box), or data was explicitly `null`.
			(None, None) => {
				debug!(
					"[Sky UI Resp Handler] ReqID '{}' resolved by Sky with no data and no error. Interpreting as user \
					 cancellation or void success.",
					request_id
				);

				// Represent cancellation or no specific data return with Value::Null.
				Ok(Value::Null)
			},
		};

		// Send the determined result back to the waiting task in `environment.rs`.
		send_ui_operation_result_to_environment_task(&request_id, oneshot_sender, result_for_environment_task);
	} else {
		// This case means the sender was not found in the map *after* the lock was
		// initially checked and released. This should be rare if the `contains_key`
		// check passed, but it's logged for robustness (e.g., if there was an
		// unexpected race condition or logic error in cleanup).
		warn!(
			"[Sky UI Resp Handler] No pending UI request sender found for ReqID '{}' after lock release (unexpected \
			 state or race condition). Sky's response for this request was ignored.",
			request_id
		);
	}

	// The Tauri command `sky_resolves_ui_request` itself completed its processing
	// successfully. The outcome of the UI operation it relayed is handled by the
	// oneshot channel.
	Ok(())
}

// Note: Specific handlers like `sky_resolves_open_dialog` are no longer
// strictly necessary if `sky_resolves_ui_request` is used generically. The
// `UiProvider` methods in `environment.rs` that initiate these UI requests are
// responsible for:
// 1. Emitting a uniquely identifiable event to Sky (e.g.,

//    `sky://ui/show-open-dialog-request`).
// 2. Storing the `oneshot::Sender` in `AppState.pending_ui_requests` keyed by
//    that unique ID.
// 3. Awaiting the `oneshot::Receiver`.
// Sky, upon completing the UI interaction, calls `sky_resolves_ui_request` with
// that same unique ID and the appropriate `data_val` or `error_details_val`.
// The `UiProvider` method then receives this `Value` (or `CommonError`) and
// parses it according to the expected return type of that specific UI operation
// (e.g., parsing `Value` into `Option<Vec<PathBuf>>` for an open dialog).
