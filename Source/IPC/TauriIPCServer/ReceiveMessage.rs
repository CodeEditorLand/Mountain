//! Validates permissions, routes an incoming Wind IPC Message to the
//! managed `TauriIPCServer` state, and records performance metrics.
//! Body of the `mountain_ipc_receive_message` command.

use tauri::Manager;

use super::{SecurityEvent, SecurityEventType, TauriIPCMessage, TauriIPCServer};
use crate::dev_log;

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// Correlation ids already acknowledged with a cancelled completion. Keeps
/// a duplicate (or late) cancel from emitting a second completion.
static CANCELLED:OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

/// Correlation ids that reached a handler. A cancel arriving for one of
/// these is warned about as "already settled" instead of double-acking.
static SETTLED:OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn Cancelled() -> &'static Mutex<HashSet<String>> {
	CANCELLED.get_or_init(|| Mutex::new(HashSet::new()))
}

fn Settled() -> &'static Mutex<HashSet<String>> {
	SETTLED.get_or_init(|| Mutex::new(HashSet::new()))
}

/// True when the incoming envelope carries the `Cancel` variant, either as
/// the typed command `Kind` or as the Cocoon `Type: "cancel"` request type.
fn IsCancelMessage(Message:&TauriIPCMessage) -> bool {
	let Kind = Message.data.get("Kind").and_then(|Value| Value.as_str());

	match Kind {
		Some("Cancel") => true,

		_ => matches!(
			Message.data.get("Type").and_then(|Value| Value.as_str()),
			Some("cancel")
		),
	}
}

/// Reuse the request's existing correlation id; never mint a parallel one.
fn CorrelationIdOf(Message:&TauriIPCMessage) -> Option<String> {
	["CorrelationId", "correlationId", "correlation_id"]
		.into_iter()
		.find_map(|Key| {
			Message
				.data
				.get(Key)
				.and_then(|Value| Value.as_str())
				.map(str::to_string)
		})
}

pub(crate) async fn Fn(app_handle:tauri::AppHandle, Message:TauriIPCMessage) -> Result<(), String> {
	dev_log!(
		"ipc",
		"[TauriIPCServer] Received IPC Message from Wind on channel: {}",
		Message.channel
	);

	// Get the IPC server instance from application state
	if let Some(ipc_server) = app_handle.try_state::<TauriIPCServer>() {
		// Envelope-level cancellation. A `Cancel` variant is keyed by the
		// request's existing CorrelationId and is acknowledged with a
		// cancelled completion instead of being dispatched to a handler.
		if IsCancelMessage(&Message) {
			let Id = match CorrelationIdOf(&Message) {
				Some(Id) if !Id.trim().is_empty() => Id,

				_ => {
					dev_log!("ipc", "warn: [TauriIPCServer] cancel message without a CorrelationId - ignoring");

					return Ok(());
				},
			};

			// Already-settled request: a cancel can no longer affect work.
			match Settled().lock() {
				Ok(Guard) if Guard.contains(&Id) => {
					dev_log!("ipc", "warn: [TauriIPCServer] cancel for {} arrived after settle - ignoring", Id);

					return Ok(());
				},

				Ok(_) => {},

				Err(e) => {
					dev_log!("ipc", "warn: [TauriIPCServer] cancel registry poisoned ({}) - degrading", e);

					return Ok(());
				},
			};

			// Duplicate cancel: only the first one emits a completion.
			match Cancelled().lock() {
				Ok(mut Guard) if !Guard.insert(Id.clone()) => {
					dev_log!("ipc", "warn: [TauriIPCServer] duplicate cancel for {} - ignoring", Id);

					return Ok(());
				},

				Ok(_) => {},

				Err(e) => {
					dev_log!("ipc", "warn: [TauriIPCServer] cancel registry poisoned ({}) - degrading", e);

					return Ok(());
				},
			};

			// This envelope layer has no per-request cancellation handle;
			// cooperative stop lives with the git/search/language handlers.
			// Warn, then still settle the caller so it cannot hang.
			dev_log!("ipc", "warn: [TauriIPCServer] cancel for {} acknowledged at the envelope; in-flight work cannot be stopped here", Id);

			let Completion = TauriIPCMessage {
				channel:Message.channel.clone(),

				data:serde_json::json!({
					"CorrelationId": Id,
					"Type": "response",
					"Success": false,
					"Error": "cancelled",
					"Cancelled": true,
					"Timestamp": std::time::SystemTime::now()
						.duration_since(std::time::UNIX_EPOCH)
						.unwrap_or_default()
						.as_millis() as u64,
				}),

				sender:Some("mountain".to_string()),

				timestamp:std::time::SystemTime::now()
					.duration_since(std::time::UNIX_EPOCH)
					.unwrap_or_default()
					.as_millis() as u64,
			};

			if let Err(e) = ipc_server.emit_message(&Completion).await {
				dev_log!("ipc", "warn: [TauriIPCServer] failed to emit cancelled completion for {}: {}", Id, e);
			}

			return Ok(());
		}

		// Advanced security: Validate permissions before processing
		if let Err(e) = ipc_server.validate_message_permissions(&Message).await {
			dev_log!(
				"ipc",
				"error: [TauriIPCServer] Permission validation failed for channel {}: {}",
				Message.channel,
				e
			);

			// Log security event
			ipc_server
				.log_security_event(SecurityEvent {
					event_type:SecurityEventType::PermissionDenied,
					user_id:Message.sender.clone().unwrap_or("unknown".to_string()),
					operation:Message.channel.clone(),
					timestamp:std::time::SystemTime::now(),
					details:Some(format!("Permission denied: {}", e)),
				})
				.await;

			return Err(format!("Permission denied: {}", e));
		}

		// Advanced monitoring: Track Message processing time
		let start_time = std::time::Instant::now();

		let result = ipc_server.IncomingMessage(Message.clone()).await;

		// Mark any correlation id that reached a handler as settled so a late
		// cancel is warned about instead of emitting a second completion.
		if let Some(Id) = CorrelationIdOf(&Message) {
			if let Ok(mut Guard) = Settled().lock() {
				Guard.insert(Id);
			}
		}

		let duration = start_time.elapsed();

		// Record performance metrics
		ipc_server
			.record_performance_metrics(Message.channel, duration, result.is_ok())
			.await;

		result
	} else {
		Err("IPC Server not found in application state".to_string())
	}
}
