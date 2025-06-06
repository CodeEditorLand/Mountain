// ---------------------------------------------------------------------------------------------
// Mountain Output Channel Handlers 
// --------------------------------------------------------------------------------------------
// Manages state and handles RPC requests related to Output Channels created by
// extensions running in sidecars (e.g., Cocoon). This allows extensions to log
// information to dedicated, named channels in the frontend (Sky).
//
// Responsibilities:
// - Handling RPC calls from Cocoon's output channel shim:
//   - `$register`: Creates a new output channel.
//   - `$append`: Appends text to an existing channel's buffer.
//   - `$clear`: Clears the buffer of a channel.
//   - `$replace`: Replaces the entire content of a channel's buffer.
//   - `$reveal`: Requests the frontend to show/focus a channel.
//   - `$close`: Informs the frontend that a channel view can be closed
//     (hidden).
//   - `$dispose`: Removes a channel and its state entirely.
// - Storing output channel state (`OutputChannelState`) in
//   `AppState.output_channels`.
// - Emitting Tauri events (e.g., `output_channel_registered`,

//   `output_channel_append`) to notify Sky about channel changes, enabling UI
//   updates.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` when an output channel related
//   RPC is received.
// - Interacts with `AppState.output_channels` (a `HashMap<String,

//   OutputChannelState>`) for state management.
// - Emits Tauri events via `AppHandle::emit` to Sky.
// - Uses `handlers::error_utils` for consistent RPC error formatting.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	// StdMutex is used for AppState.output_channels
	sync::MutexGuard,
};

use log::{error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Emitter, Manager, Runtime};

use crate::{
	app_state::{AppState, OutputChannelState},

	// Use shared error utilities
	handlers::error_utils,
};

/// Helper to acquire a lock on the `output_channels` map in `AppState`.
///
/// Handles potential `PoisonError` by converting it to a formatted RPC error
/// string.
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
///
/// # Returns
/// * `Ok(MutexGuard)` to the output channels map.
/// * `Err(String)` containing a JSON-RPC error string if the lock is poisoned.
fn get_output_channels_map_lock<'a, R:Runtime>(
	app:&'a AppHandle<R>,
) -> Result<MutexGuard<'a, HashMap<String, OutputChannelState>>, String> {
	let state = app.state::<AppState>();

	state.output_channels.lock().map_err(|e| {
		let msg = format!("Output channels lock is poisoned: {}", e);

		// Keep specific error log for internal diagnostics
		error!("[Output Handler LockErr] {}", msg);

		// Use a specific error code for output channel lock issues
		error_utils::rpc_error_string(msg, Some("ELOCKED_OUTPUT"))
	})
}

/// Handles the `$register` RPC call from Cocoon's output channel shim.
///
/// Creates a new output channel state entry in `AppState`. The channel ID is
/// currently the same as its name.
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[name: string, file?: URI | null,
///
///   languageId?: string | null]`
///   - `name`: The human-readable name of the output channel.
///   - `file` (optional): URI of a file to associate with the channel
///     (currently unused in MVP).
///   - `languageId` (optional): Language ID for syntax highlighting in the
///     channel (e.g., "log").
///
/// # Returns
/// * `Ok(Value::String)` containing the ID of the registered channel (which is
///   its name).
/// * `Err(String)` with a JSON-RPC error if parsing or registration fails.
pub async fn handle_register_output_channel<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let name = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_register", "name", "string", Some(0)))?
		.to_string();

	// `file` (args[1]) is an optional URI for a file to associate with the channel.
	// Not used in MVP's OutputChannelState, but could be in the future.
	let _file_uri_val = args.get(1);

	trace!("[Output Handler] Register: file_uri (args[1]) = {:?}", _file_uri_val);

	let language_id = args.get(2).and_then(Value::as_str).map(String::from);

	// Keep: Registration is an important log event.
	info!(
		"[Output Handler] Registering output channel: name='{}', languageId={:?}",
		name, language_id
	);

	// For MVP, the channel ID is the same as its display name.
	// TODO: Consider generating unique IDs if names might collide or change.
	let channel_id = name.clone();

	{
		// Scope for Mutex lock
		let mut channels_state_map_guard = get_output_channels_map_lock(&app)?;

		// `or_insert_with` creates and inserts if not present, or returns a mutable
		// ref if present. This ensures that registering an existing channel name is
		// idempotent for state creation, though a new event will still be emitted.
		channels_state_map_guard
			.entry(channel_id.clone())
			.or_insert_with(|| OutputChannelState::new(&name, language_id));

		// Lock released here
	}

	// Notify Sky (frontend) that a new channel is available.
	let event_payload = json!({"id": channel_id, "name": name});

	// Keep: Log event emission for traceability.
	trace!(
		"[Output Handler] Emitting 'output_channel_registered' event: {:?}",
		event_payload
	);

	app.emit("output_channel_registered", event_payload).map_err(|e| {
		let msg = format!("Failed to emit output_channel_registered event: {}", e);

		error!("[Output Handler] {}", msg);

		error_utils::rpc_error_string(msg, Some("EEMIT_OCHANNEL_REG"))
	})?;

	// Return the channel ID (name) to Cocoon.
	Ok(json!(channel_id))
}

/// Handles the `$append` RPC call.
///
/// Appends the given `value` string to the buffer of the specified output
/// channel.
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[channelId: string, value: string]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)` if the channel ID is not found or parameters are invalid.
pub async fn handle_append_to_output_channel<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_append", "channelId", "string", Some(0)))?;

	// Default to empty string if value is missing/not string
	let value_to_append = args.get(1).and_then(Value::as_str).unwrap_or("");

	// Reduce logging verbosity for frequent append operations, use trace.
	trace!(
		"[Output Handler] Appending to channel '{}': len={}",
		channel_id,
		value_to_append.len()
	);

	if value_to_append.is_empty() {
		// No-op if appending empty string
		return Ok(Value::Null);
	}

	let mut event_payload_opt:Option<Value> = None;

	{
		let mut channels_state_map_guard = get_output_channels_map_lock(&app)?;

		if let Some(channel_state) = channels_state_map_guard.get_mut(channel_id) {
			channel_state.buffer.push_str(value_to_append);

			// TODO: Consider limiting total buffer size per channel to prevent excessive
			// memory usage. If limit is reached, could truncate from the beginning.
			// e.g., if channel_state.buffer.len() > MAX_BUFFER_SIZE { channel_state.buffer
			// = ... }

			event_payload_opt =
				Some(json!({"id": channel_id.to_string(), "appendedText": value_to_append.to_string()}));
		} else {
			warn!(
				"[Output Handler] Output channel '{}' not found for append operation.",
				channel_id
			);

			// VS Code behavior: if channel doesn't exist, append is a no-op.
			// Alternatively, could return an error:
			// return Err(error_utils::rpc_error_string(format!("Channel '{}'
			// not found", channel_id), Some("ENOCHANNEL")));
		}

		// Lock released
	}

	if let Some(payload) = event_payload_opt {
		// This can be very noisy if logged at info/debug for every append. Use trace.
		trace!(
			"[Output Handler] Emitting 'output_channel_append': id={}, len={}",
			channel_id,
			value_to_append.len()
		);

		app.emit("output_channel_append", payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_append event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT_OCHANNEL_APPEND"))
		})?;
	}

	Ok(Value::Null)
}

/// Handles the `$clear` RPC call.
///
/// Clears the entire buffer content of the specified output channel.
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[channelId: string]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)` if the channel ID is not found or parameters are invalid.
pub async fn handle_clear_output_channel<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_clear", "channelId", "string", Some(0)))?;

	// Keep: Clear is a distinct, less frequent user-visible action.
	info!("[Output Handler] Clearing output channel: '{}'", channel_id);

	let mut event_needed = false;

	{
		let mut channels_state_map_guard = get_output_channels_map_lock(&app)?;

		if let Some(channel_state) = channels_state_map_guard.get_mut(channel_id) {
			if !channel_state.buffer.is_empty() {
				channel_state.buffer.clear();

				// Only emit event if content was actually cleared.
				event_needed = true;
			}
		} else {
			warn!(
				"[Output Handler] Output channel '{}' not found for clear operation.",
				channel_id
			);

			// VS Code: no-op if channel doesn't exist.
		}

		// Lock released
	}

	if event_needed {
		let event_payload = json!({"id": channel_id.to_string()});

		// Keep: Log event emission.
		trace!("[Output Handler] Emitting 'output_channel_clear' event: {:?}", event_payload);

		app.emit("output_channel_clear", event_payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_clear event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT_OCHANNEL_CLEAR"))
		})?;
	}

	Ok(Value::Null)
}

/// Handles the `$replace` RPC call.
///
/// Replaces the entire buffer content of the specified output channel with the
/// new `value`.
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[channelId: string, value: string]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)` if the channel ID is not found or parameters are invalid.
pub async fn handle_replace_output_channel_content<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_replace", "channelId", "string", Some(0)))?;

	// Ensure new_value is an owned String.
	let new_value = args.get(1).and_then(Value::as_str).unwrap_or("").to_string();

	// Keep: Replace is a distinct action.
	info!(
		"[Output Handler] Replacing content of output channel: '{}' (new length: {})",
		channel_id,
		new_value.len()
	);

	let mut event_payload_opt:Option<Value> = None;

	{
		let mut channels_state_map_guard = get_output_channels_map_lock(&app)?;

		if let Some(channel_state) = channels_state_map_guard.get_mut(channel_id) {
			// Use cloned new_value
			channel_state.buffer = new_value.clone();

			event_payload_opt = Some(json!({"id": channel_id.to_string(), "fullContent": new_value}));
		} else {
			warn!(
				"[Output Handler] Output channel '{}' not found for replace operation.",
				channel_id
			);

			// VS Code: no-op if channel doesn't exist.
		}

		// Lock released
	}

	if let Some(payload) = event_payload_opt {
		// Keep: Log event emission.
		trace!("[Output Handler] Emitting 'output_channel_replace' event: id={}", channel_id);

		app.emit("output_channel_replace", payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_replace event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT_OCHANNEL_REPLACE"))
		})?;
	}

	Ok(Value::Null)
}

/// Handles the `$reveal` RPC call.
///
/// Requests the frontend (Sky) to show and potentially focus the specified
/// output channel. Mountain updates its internal state for the channel's
/// visibility and emits an event.
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[channelId: string, preserveFocus:
///   boolean]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)` if the channel ID is not found or parameters are invalid.
pub async fn handle_reveal_output_channel<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_reveal", "channelId", "string", Some(0)))?;

	let preserve_focus = args.get(1).and_then(Value::as_bool).unwrap_or(false);

	// Keep: UI action log.
	info!(
		"[Output Handler] Revealing output channel: '{}', preserveFocus={}",
		channel_id, preserve_focus
	);

	let mut event_needed = false;

	{
		let mut channels_state_map_guard = get_output_channels_map_lock(&app)?;

		if let Some(channel_state) = channels_state_map_guard.get_mut(channel_id) {
			// Update internal visibility state.
			// This helps if Mountain needs to know which channels are "active".
			channel_state.visible = true;

			event_needed = true;
		} else {
			warn!(
				"[Output Handler] Output channel '{}' not found for reveal operation.",
				channel_id
			);

			// VS Code: no-op if channel doesn't exist.
		}

		// Lock released
	}

	if event_needed {
		let event_payload = json!({"id": channel_id.to_string(), "preserveFocus": preserve_focus });

		// Keep: Log event emission.
		trace!("[Output Handler] Emitting 'output_channel_reveal' event: {:?}", event_payload);

		app.emit("output_channel_reveal", event_payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_reveal event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT_OCHANNEL_REVEAL"))
		})?;
	}

	Ok(Value::Null)
}

/// Handles the `$close` RPC call.
///
/// Informs the frontend (Sky) that the view for the specified output channel
/// can be closed (hidden). Mountain updates its internal visibility state.
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[channelId: string]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)` if the channel ID is not found or parameters are invalid.
pub async fn handle_close_output_channel_view<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_close", "channelId", "string", Some(0)))?;

	// Keep: UI action log.
	info!("[Output Handler] Closing output channel view requested for: '{}'", channel_id);

	let mut event_needed = false;

	{
		let mut channels_state_map_guard = get_output_channels_map_lock(&app)?;

		if let Some(channel_state) = channels_state_map_guard.get_mut(channel_id) {
			if channel_state.visible {
				// Only update and emit if it was previously marked as visible.
				channel_state.visible = false;

				event_needed = true;
			}
		} else {
			warn!(
				"[Output Handler] Channel '{}' not found for close (view) operation (maybe already disposed).",
				channel_id
			);

			// VS Code: no-op if channel doesn't exist.
		}

		// Lock released
	}

	if event_needed {
		let event_payload = json!({"id": channel_id.to_string() });

		// Keep: Log event emission.
		trace!("[Output Handler] Emitting 'output_channel_close' event: {:?}", event_payload);

		app.emit("output_channel_close", event_payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_close event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT_OCHANNEL_CLOSE"))
		})?;
	}

	Ok(Value::Null)
}

/// Handles the `$dispose` RPC call.
///
/// Removes the output channel and its associated state entirely from the
/// backend (`AppState`).
///
/// # Argument
/// * `app` - The Tauri `AppHandle`.
/// * `args` - A `serde_json::Value` array: `[channelId: string]`
///
/// # Returns
/// * `Ok(Value::Null)` on success.
/// * `Err(String)` if parameters are invalid or an internal error occurs.
pub async fn handle_dispose_output_channel<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_dispose", "channelId", "string", Some(0)))?;

	// Keep: Lifecycle event log.
	info!("[Output Handler] Disposing output channel: '{}'", channel_id);

	let mut event_needed = false;

	{
		let mut channels_state_map_guard = get_output_channels_map_lock(&app)?;

		if channels_state_map_guard.remove(channel_id).is_some() {
			info!("[Output Handler] Disposed channel '{}' state from AppState.", channel_id);

			event_needed = true;
		} else {
			warn!(
				"[Output Handler] Channel '{}' not found for dispose operation (maybe already disposed).",
				channel_id
			);

			// VS Code: no-op if channel doesn't exist.
		}

		// Lock released
	}

	if event_needed {
		let event_payload = json!({"id": channel_id.to_string()});

		// Keep: Log event emission.
		trace!("[Output Handler] Emitting 'output_channel_disposed' event: {:?}", event_payload);

		app.emit("output_channel_disposed", event_payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_disposed event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT_OCHANNEL_DISPOSE"))
		})?;
	}

	Ok(Value::Null)
}

// NEW:
// // Example signature in Handler/output.rs
// pub async fn handle_register_output_channel_effect_logic<R: tauri::Runtime>(
//     app_handle: tauri::AppHandle<R>,
//     name: String,
//     language_id: Option<String>,
// ) -> Result<String, CommonError> {
//     // ... implementation using AppState and emitting Tauri events ...
//     todo!()
// }

// pub async fn handle_append_to_output_channel_effect_logic<R: tauri::Runtime>(
//     app_handle: tauri::AppHandle<R>,
//     channel_id: String,
//     value: String,
// ) -> Result<(), CommonError> {
//     // ... implementation ...
//     todo!()
// }
// // ... and so on for replace, clear, reveal, close, dispose ...
