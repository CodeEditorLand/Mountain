// ---------------------------------------------------------------------------------------------
// Mountain Output Channel Handlers (handlers/output.rs)
// --------------------------------------------------------------------------------------------
// Manages state and handles RPC requests related to Output Channels created by
// extensions running in sidecars (e.g., Cocoon).
//
// Responsibilities:
// - Handling `$register` RPC calls: Records the existence of a new output
//   channel, storing its state (buffer, visibility) in `AppState`.
// - Handling `$append`, `$clear`, `$replace` RPC calls: Modifies the buffer
//   associated with a specific channel ID (stored in `AppState`).
// - Handling `$reveal`, `$close` RPC calls: Manages the visibility state of the
//   channel.
// - Handling `$dispose` RPC calls: Removes the channel's state from `AppState`.
// - Emitting Tauri events (e.g., `output_channel_append`,

//   `output_channel_reveal`) to notify the frontend (Sky) about changes,

//   allowing the UI Output panel to update.
//
// Key Interactions:
// - Called by `track::dispatch_sidecar_request` for RPC methods.
// - Interacts with `AppState` via Mutex to manage the `output_channels` map.
// - Emits Tauri events via `AppHandle::emit_all` to communicate with the
//   frontend UI.
// --------------------------------------------------------------------------------------------

use std::{
	collections::HashMap,

	// Use StdMutex if AppState uses it directly
	sync::{Arc, Mutex as StdMutex, MutexGuard},
};

// Use log crate
use log;
use serde_json::{Value, json};
use tauri::{AppHandle, Manager, Runtime};

// Import state struct and the AppState itself
use crate::app_state::{AppState, OutputChannelState};

/// Helper to get a locked mutable reference to the output channels map in
/// AppState. Handles potential lock poisoning.
fn get_output_channels_lock<'a, R:Runtime>(
	app:&'a AppHandle<R>,
) -> Result<MutexGuard<'a, HashMap<String, OutputChannelState>>, String> {
	let state = app.state::<AppState>();

	state.output_channels.lock().map_err(|e| {
		log::error!("Output channels lock is poisoned: {}", e);

		// Return error string
		format!("Failed to lock output channels state: {}", e)
	})
}

/// Handles the `$register` RPC call.
/// Creates a new output channel state entry.
/// Args: `[name: string, file?: URI | null, languageId?: string | null]`
pub async fn handle_register<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let name = args
		.get(0)
		.and_then(|v| v.as_str())
		.ok_or_else(|| "Missing or invalid 'name' (string) argument".to_string())?
		.to_string();

	let language_id = args.get(2).and_then(|v| v.as_str()).map(|s| s.to_string());

	// TODO: Handle file URI (args[1]) if provided for file-backed channels
	// Keep: Registration is important log
	log::info!("[Output Handler] Register channel: name='{}', langId={:?}", name, language_id);

	let mut channels_state = get_output_channels_lock(&app)?;

	// Use name as ID for MVP
	let channel_id = name.clone();

	channels_state
		.entry(channel_id.clone())
		.or_insert_with(|| OutputChannelState::new(&name, language_id));

	// Drop lock before emit
	drop(channels_state);

	// Keep: Log event emission
	let event_payload = json!({"id": channel_id, "name": name});

	log::trace!("[Output Handler] Emitting output_channel_registered event: {:?}", event_payload);

	app.emit_all("output_channel_registered", event_payload)
		.map_err(|e| log::error!("Failed to emit output_channel_registered event: {}", e))
		.ok();

	// Return the ID used
	Ok(json!(channel_id))
}

/// Handles the `$append` RPC call.
/// Appends text to the specified channel's buffer.
/// Args: `[channelId: string, value: string]`
pub async fn handle_append<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(|v| v.as_str())
		.ok_or_else(|| "Missing or invalid 'channelId' (string) argument".to_string())?;

	let value = args.get(1).and_then(|v| v.as_str()).unwrap_or("");

	// Reduce logging verbosity for append
	log::trace!("[Output Handler] Append to '{}': len={}", channel_id, value.len());

	let mut channels_state = get_output_channels_lock(&app)?;

	if let Some(channel) = channels_state.get_mut(channel_id) {
		channel.buffer.push_str(value);

		// TODO: Consider limiting total buffer size

		let id_clone = channel_id.to_string();

		// Clone value *before* dropping lock if needed by event
		let value_clone = value.to_string();

		// Drop lock before emitting event
		drop(channels_state);

		// Keep: Log event emission
		let event_payload = json!({"id": id_clone, "value": value_clone});

		// log::trace!("[Output Handler] Emitting output_channel_append event: {:?}",

		// Can be noisy
		// event_payload);

		app.emit_all("output_channel_append", event_payload)
			.map_err(|e| log::error!("Failed to emit output_channel_append event: {}", e))
			.ok();
	} else {
		log::warn!("[Output Handler] Output channel '{}' not found for append.", channel_id);

		// Ensure lock is dropped on error path too
		drop(channels_state);
	}
	// Void operation
	Ok(Value::Null)
}

/// Handles the `$clear` RPC call.
/// Clears the buffer of the specified channel.
/// Args: `[channelId: string]`
pub async fn handle_clear<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(|v| v.as_str())
		.ok_or_else(|| "Missing or invalid 'channelId' (string) argument".to_string())?;

	// Keep: Clear is a distinct action
	log::info!("[Output Handler] Clear channel: '{}'", channel_id);

	let mut channels_state = get_output_channels_lock(&app)?;

	if let Some(channel) = channels_state.get_mut(channel_id) {
		channel.buffer.clear();

		let id_clone = channel_id.to_string();

		// Drop lock before emitting event
		drop(channels_state);

		// Keep: Log event emission
		let event_payload = json!({"id": id_clone});

		log::trace!("[Output Handler] Emitting output_channel_clear event: {:?}", event_payload);

		app.emit_all("output_channel_clear", event_payload)
			.map_err(|e| log::error!("Failed to emit output_channel_clear event: {}", e))
			.ok();
	} else {
		log::warn!("[Output Handler] Output channel '{}' not found for clear.", channel_id);

		drop(channels_state);
	}
	// Void operation
	Ok(Value::Null)
}

/// Handles the `$replace` RPC call.
/// Replaces the entire buffer content of the specified channel.
/// Args: `[channelId: string, value: string]`
pub async fn handle_replace<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(|v| v.as_str())
		.ok_or_else(|| "Missing or invalid 'channelId' (string) argument".to_string())?;

	let value = args.get(1).and_then(|v| v.as_str()).unwrap_or("");

	// Keep: Replace is a distinct action
	log::info!("[Output Handler] Replace channel: '{}'", channel_id);

	let mut channels_state = get_output_channels_lock(&app)?;

	if let Some(channel) = channels_state.get_mut(channel_id) {
		channel.buffer = value.to_string();

		let id_clone = channel_id.to_string();

		let value_clone = value.to_string();

		// Drop lock before emitting event
		drop(channels_state);

		// Keep: Log event emission
		let event_payload = json!({"id": id_clone, "value": value_clone});

		log::trace!("[Output Handler] Emitting output_channel_replace event: {:?}", event_payload);

		app.emit_all("output_channel_replace", event_payload)
			.map_err(|e| log::error!("Failed to emit output_channel_replace event: {}", e))
			.ok();
	} else {
		log::warn!("[Output Handler] Output channel '{}' not found for replace.", channel_id);

		drop(channels_state);
	}
	// Void operation
	Ok(Value::Null)
}

/// Handles the `$reveal` RPC call.
/// Requests the frontend to show the specified output channel.
/// Args: `[channelId: string, preserveFocus: boolean]`
pub async fn handle_reveal<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(|v| v.as_str())
		.ok_or_else(|| "Missing or invalid 'channelId' (string) argument".to_string())?;

	let preserve_focus = args.get(1).and_then(|v| v.as_bool()).unwrap_or(false);

	// Keep: UI action log
	log::info!(
		"[Output Handler] Reveal channel: '{}', preserveFocus={}",
		channel_id,
		preserve_focus
	);

	let mut channels_state = get_output_channels_lock(&app)?;

	if let Some(channel) = channels_state.get_mut(channel_id) {
		// Update internal state
		channel.visible = true;

		let id_clone = channel_id.to_string();

		// Drop lock before emitting event
		drop(channels_state);

		// Keep: Log event emission
		let event_payload = json!({"id": id_clone, "preserveFocus": preserve_focus });

		log::trace!("[Output Handler] Emitting output_channel_reveal event: {:?}", event_payload);

		app.emit_all("output_channel_reveal", event_payload)
			.map_err(|e| log::error!("Failed to emit output_channel_reveal event: {}", e))
			.ok();
	} else {
		log::warn!("[Output Handler] Output channel '{}' not found for reveal.", channel_id);

		drop(channels_state);
	}
	// Void operation
	Ok(Value::Null)
}

/// Handles the `$close` RPC call.
/// Informs the frontend that the channel view can be closed.
/// Args: `[channelId: string]`
pub async fn handle_close<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(|v| v.as_str())
		.ok_or_else(|| "Missing or invalid 'channelId' (string) argument".to_string())?;

	// Keep: UI action log
	log::info!("[Output Handler] Close channel requested: '{}'", channel_id);

	let mut channels_state = get_output_channels_lock(&app)?;

	if let Some(channel) = channels_state.get_mut(channel_id) {
		// Update internal state
		channel.visible = false;

		let id_clone = channel_id.to_string();

		// Drop lock before emitting event
		drop(channels_state);

		// Keep: Log event emission
		let event_payload = json!({"id": id_clone });

		log::trace!("[Output Handler] Emitting output_channel_close event: {:?}", event_payload);

		app.emit_all("output_channel_close", event_payload)
			.map_err(|e| log::error!("Failed to emit output_channel_close event: {}", e))
			.ok();
	} else {
		log::warn!(
			"[Output Handler] Channel '{}' not found for close (maybe already disposed).",
			channel_id
		);

		drop(channels_state);
	}
	// Void operation
	Ok(Value::Null)
}

/// Handles the `$dispose` RPC call.
/// Removes the channel state entirely from the backend.
/// Args: `[channelId: string]`
pub async fn handle_dispose<R:Runtime>(app:AppHandle<R>, args:Value) -> Result<Value, String> {
	let channel_id = args
		.get(0)
		.and_then(|v| v.as_str())
		.ok_or_else(|| "Missing or invalid 'channelId' (string) argument".to_string())?;

	// Keep: Lifecycle event log
	log::info!("[Output Handler] Dispose channel: '{}'", channel_id);

	let mut channels_state = get_output_channels_lock(&app)?;

	if channels_state.remove(channel_id).is_some() {
		log::info!("[Output Handler] Disposed channel '{}' state.", channel_id);

		let id_clone = channel_id.to_string();

		// Drop lock before emitting event
		drop(channels_state);

		// Keep: Log event emission
		let event_payload = json!({"id": id_clone });

		log::trace!("[Output Handler] Emitting output_channel_disposed event: {:?}", event_payload);

		app.emit_all("output_channel_disposed", event_payload)
			.map_err(|e| log::error!("Failed to emit output_channel_disposed event: {}", e))
			.ok();
	} else {
		log::warn!(
			"[Output Handler] Channel '{}' not found for dispose (maybe already disposed).",
			channel_id
		);

		drop(channels_state);
	}
	// Void operation
	Ok(Value::Null)
}
