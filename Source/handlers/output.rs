// ---------------------------------------------------------------------------------------------
// Mountain Output Channel Handlers (handlers/output.rs)
// --------------------------------------------------------------------------------------------
// Manages state and handles RPC requests related to Output Channels created by
// extensions running in sidecars (e.g., Cocoon).
//
// Responsibilities:
// - Handling RPC calls: $register, $append, $clear, $replace, $reveal, $close,
//   $dispose.
// - Storing output channel state in `AppState`.
// - Emitting Tauri events to notify Sky about channel changes.
// Key Interactions:
// - Called by `track::dispatch_sidecar_request`.
// - Interacts with `AppState` to manage `output_channels` map.
// - Emits Tauri events via `AppHandle::emit_all`.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	// StdMutex used if AppState field is direct
	sync::{Arc, Mutex as StdMutex, MutexGuard},
};

use log::{debug, error, info, trace, warn};
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};

use crate::{
	app_state::{AppState, OutputChannelState},

	handlers::error_utils,
	// Use shared error utilities
};

/// Helper to get a locked mutable reference to the output channels map in
/// AppState.
fn get_output_channels_lock<'a, R:Runtime>(
	app:&'a AppHandle<R>,
) -> Result<MutexGuard<'a, HashMap<String, OutputChannelState>>, String> {
	let state = app.state::<AppState>();

	state.output_channels.lock().map_err(|e| {
		let msg = format!("Output channels lock is poisoned: {}", e);

		// Keep specific error log
		error!("[Output Handler LockErr] {}", msg);

		// Use a specific code if desired
		error_utils::rpc_error_string(msg, Some("ELOCKED_OUTPUT"))
	})
}

/// Handles the `$register` RPC call.
/// Creates a new output channel state entry.
/// Args: `[name: string, file?: URI | null, languageId?: string | null]`
pub async fn handle_register<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let name = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_register", "name", "string", Some(0)))?
		.to_string();

	let language_id = args.get(2).and_then(Value::as_str).map(String::from);

	// args[1] is file_uri (Option<UriComponentsDTO>), currently unused by
	// OutputChannelState::new for MVP.

	// Keep: Registration is important log
	info!("[Output Handler] Register channel: name='{}', langId={:?}", name, language_id);

	// For MVP, channel ID is the name
	let channel_id = name.clone();

	{
		// Scope for lock
		let mut channels_state = get_output_channels_lock(&app)?;

		channels_state
			.entry(channel_id.clone())
			.or_insert_with(|| OutputChannelState::new(&name, language_id));

		// Lock released
	}

	let event_payload = json!({"id": channel_id, "name": name});

	// Keep: Log event emission
	trace!("[Output Handler] Emitting output_channel_registered event: {:?}", event_payload);

	app.emit_all("output_channel_registered", event_payload).map_err(|e| {
		let msg = format!("Failed to emit output_channel_registered event: {}", e);

		error!("[Output Handler] {}", msg);

		error_utils::rpc_error_string(msg, Some("EEMIT"))
	})?;

	Ok(json!(channel_id))
}

/// Handles the `$append` RPC call.
/// Appends text to the specified channel's buffer.
/// Args: `[channelId: string, value: string]`
pub async fn handle_append<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_append", "channelId", "string", Some(0)))?;

	let value_to_append = args.get(1).and_then(Value::as_str).unwrap_or("");

	// Reduce logging verbosity for append
	trace!("[Output Handler] Append to '{}': len={}", channel_id, value_to_append.len());

	let mut event_payload_opt:Option<Value> = None;

	{
		// Scope for lock
		let mut channels_state = get_output_channels_lock(&app)?;

		if let Some(channel) = channels_state.get_mut(channel_id) {
			channel.buffer.push_str(value_to_append);

			// TODO: Consider limiting total buffer size per channel
			event_payload_opt = Some(json!({"id": channel_id.to_string(), "value": value_to_append.to_string()}));
		} else {
			warn!("[Output Handler] Output channel '{}' not found for append.", channel_id);
		}

		// Lock released
	}

	if let Some(payload) = event_payload_opt {
		// trace!("[Output Handler] Emitting output_channel_append: {:?}", payload); //
		// This can be very noisy
		app.emit_all("output_channel_append", payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_append event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT"))
		})?;
	}

	Ok(Value::Null)
}

/// Handles the `$clear` RPC call.
/// Clears the buffer of the specified channel.
/// Args: `[channelId: string]`
pub async fn handle_clear<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_clear", "channelId", "string", Some(0)))?;

	// Keep: Clear is a distinct action
	info!("[Output Handler] Clear channel: '{}'", channel_id);

	let mut event_needed = false;

	{
		// Scope for lock
		let mut channels_state = get_output_channels_lock(&app)?;

		if let Some(channel) = channels_state.get_mut(channel_id) {
			if !channel.buffer.is_empty() {
				// Only clear and emit if there's content
				channel.buffer.clear();

				event_needed = true;
			}
		} else {
			warn!("[Output Handler] Output channel '{}' not found for clear.", channel_id);
		}

		// Lock released
	}

	if event_needed {
		let event_payload = json!({"id": channel_id.to_string()});

		// Keep: Log event emission
		trace!("[Output Handler] Emitting output_channel_clear event: {:?}", event_payload);

		app.emit_all("output_channel_clear", event_payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_clear event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT"))
		})?;
	}

	Ok(Value::Null)
}

/// Handles the `$replace` RPC call.
/// Replaces the entire buffer content of the specified channel.
/// Args: `[channelId: string, value: string]`
pub async fn handle_replace<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_replace", "channelId", "string", Some(0)))?;

	// Ensure it's owned
	let new_value = args.get(1).and_then(Value::as_str).unwrap_or("").to_string();

	// Keep: Replace is a distinct action
	info!("[Output Handler] Replace channel: '{}'", channel_id);

	let mut event_payload_opt:Option<Value> = None;

	{
		// Scope for lock
		let mut channels_state = get_output_channels_lock(&app)?;

		if let Some(channel) = channels_state.get_mut(channel_id) {
			// Use cloned new_value
			channel.buffer = new_value.clone();

			event_payload_opt = Some(json!({"id": channel_id.to_string(), "value": new_value}));
		} else {
			warn!("[Output Handler] Output channel '{}' not found for replace.", channel_id);
		}

		// Lock released
	}

	if let Some(payload) = event_payload_opt {
		// Keep: Log event emission
		trace!("[Output Handler] Emitting output_channel_replace event: {:?}", payload);

		app.emit_all("output_channel_replace", payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_replace event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT"))
		})?;
	}

	Ok(Value::Null)
}

/// Handles the `$reveal` RPC call.
/// Requests the frontend to show the specified output channel.
/// Args: `[channelId: string, preserveFocus: boolean]`
pub async fn handle_reveal<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_reveal", "channelId", "string", Some(0)))?;

	let preserve_focus = args.get(1).and_then(Value::as_bool).unwrap_or(false);

	// Keep: UI action log
	info!(
		"[Output Handler] Reveal channel: '{}', preserveFocus={}",
		channel_id, preserve_focus
	);

	let mut event_needed = false;

	{
		// Scope for lock
		let mut channels_state = get_output_channels_lock(&app)?;

		if let Some(channel) = channels_state.get_mut(channel_id) {
			// Update internal state
			channel.visible = true;

			event_needed = true;
		} else {
			warn!("[Output Handler] Output channel '{}' not found for reveal.", channel_id);
		}

		// Lock released
	}

	if event_needed {
		let event_payload = json!({"id": channel_id.to_string(), "preserveFocus": preserve_focus });

		// Keep: Log event emission
		trace!("[Output Handler] Emitting output_channel_reveal event: {:?}", event_payload);

		app.emit_all("output_channel_reveal", event_payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_reveal event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT"))
		})?;
	}

	Ok(Value::Null)
}

/// Handles the `$close` RPC call.
/// Informs the frontend that the channel view can be closed.
/// Args: `[channelId: string]`
pub async fn handle_close<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_close", "channelId", "string", Some(0)))?;

	// Keep: UI action log
	info!("[Output Handler] Close channel requested: '{}'", channel_id);

	let mut event_needed = false;

	{
		// Scope for lock
		let mut channels_state = get_output_channels_lock(&app)?;

		if let Some(channel) = channels_state.get_mut(channel_id) {
			if channel.visible {
				// Only update and emit if it was visible
				// Update internal state
				channel.visible = false;

				event_needed = true;
			}
		} else {
			warn!(
				"[Output Handler] Channel '{}' not found for close (maybe already disposed).",
				channel_id
			);
		}

		// Lock released
	}

	if event_needed {
		let event_payload = json!({"id": channel_id.to_string() });

		// Keep: Log event emission
		trace!("[Output Handler] Emitting output_channel_close event: {:?}", event_payload);

		app.emit_all("output_channel_close", event_payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_close event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT"))
		})?;
	}

	Ok(Value::Null)
}

/// Handles the `$dispose` RPC call.
/// Removes the channel state entirely from the backend.
/// Args: `[channelId: string]`
pub async fn handle_dispose<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(Value::as_str)
		.ok_or_else(|| error_utils::rpc_param_error_string("output_dispose", "channelId", "string", Some(0)))?;

	// Keep: Lifecycle event log
	info!("[Output Handler] Dispose channel: '{}'", channel_id);

	let mut event_needed = false;

	{
		// Scope for lock
		let mut channels_state = get_output_channels_lock(&app)?;

		if channels_state.remove(channel_id).is_some() {
			info!("[Output Handler] Disposed channel '{}' state.", channel_id);

			event_needed = true;
		} else {
			warn!(
				"[Output Handler] Channel '{}' not found for dispose (maybe already disposed).",
				channel_id
			);
		}

		// Lock released
	}

	if event_needed {
		let event_payload = json!({"id": channel_id.to_string()});

		// Keep: Log event emission
		trace!("[Output Handler] Emitting output_channel_disposed event: {:?}", event_payload);

		app.emit_all("output_channel_disposed", event_payload).map_err(|e| {
			let msg = format!("Failed to emit output_channel_disposed event: {}", e);

			error!("[Output Handler] {}", msg);

			error_utils::rpc_error_string(msg, Some("EEMIT"))
		})?;
	}

	Ok(Value::Null)
}
