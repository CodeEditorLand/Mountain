//! Validates permissions, routes an incoming Wind IPC Message to the
//! managed `TauriIPCServer` state, and records performance metrics.
//! Body of the `mountain_ipc_receive_message` command.

use tauri::Manager;

use super::{SecurityEvent, SecurityEventType, TauriIPCMessage, TauriIPCServer};
use crate::dev_log;

use std::collections::HashSet;

/// D27: hard ceiling on the per-server cancel/settle tombstones. The two
/// removal sites (settle and cancel-consumption) keep the working set small;
/// this cap guarantees the sets never grow without bound even when ids are
/// never reused. On overflow the registry is cleared with a warning, so
/// duplicate detection degrades instead of the IPC server.
const REGISTRY_LIMIT:usize = 4096;

/// D27: scope key for the cancel/settle registry. The message channel is the
/// per-session discriminator present on every envelope; pairing it with the
/// caller-supplied correlation id means two sessions that reuse the same id can
/// no longer see each other's tombstones. A blank channel cannot be scoped:
/// warn and return `None` so the caller degrades rather than poisoning a
/// shared entry.
fn RegistryKey(Message:&TauriIPCMessage, Id:&str) -> Option<(String, String)> {
	if Message.channel.trim().is_empty() {
		dev_log!("ipc", "warn: [TauriIPCServer] registry key has no channel - degrading without session scope");

		return None;
	}

	Some((Message.channel.clone(), Id.to_string()))
}

/// D27: warn and clear a registry that reached `REGISTRY_LIMIT` so it stays
/// bounded. Late/duplicate-cancel detection degrades for a single window; the
/// IPC server itself never panics.
fn BoundRegistry(Name:&str, Guard:&mut HashSet<(String, String)>) {
	if Guard.len() >= REGISTRY_LIMIT {
		dev_log!("ipc", "warn: [TauriIPCServer] {} registry reached {} entries - clearing to stay bounded", Name, REGISTRY_LIMIT);

		Guard.clear();
	}
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

			// D27: scope key = (channel, correlation id). A blank channel
			// yields None: skip the registry checks but still emit the
			// completion so the caller cannot hang (graceful degradation).
			let Key = RegistryKey(&Message, &Id);

			// Already-settled request: a cancel can no longer affect work.
			if let Some(ref Key) = Key {
				let mut Guard = match ipc_server.settled_registry.lock() {
					Ok(Guard) => Guard,

					Err(e) => {
						dev_log!("ipc", "warn: [TauriIPCServer] settled registry poisoned ({}) - degrading", e);

						return Ok(());
					},
				};

				if Guard.contains(Key) {
					// Consume the tombstone: a genuinely late cancel has now
					// been handled, so the entry need not live forever.
					Guard.remove(Key);

					dev_log!("ipc", "warn: [TauriIPCServer] cancel for {} arrived after settle - ignoring", Id);

					return Ok(());
				}
			}

			// Duplicate cancel: only the first one emits a completion.
			if let Some(ref Key) = Key {
				let mut Guard = match ipc_server.cancelled_registry.lock() {
					Ok(Guard) => Guard,

					Err(e) => {
						dev_log!("ipc", "warn: [TauriIPCServer] cancel registry poisoned ({}) - degrading", e);

						return Ok(());
					},
				};

				if Guard.contains(Key) {
					dev_log!("ipc", "warn: [TauriIPCServer] duplicate cancel for {} - ignoring", Id);

					return Ok(());
				}

				BoundRegistry("cancel", &mut *Guard);
				Guard.insert((*Key).clone());
			}

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

		// D27: mark this (channel, correlation id) as settled on the owning
		// server, and clear any stale cancel tombstone for the same key so a
		// caller reusing an id within the channel is not suppressed by a prior
		// cancel.
		if let Some(Id) = CorrelationIdOf(&Message) {
			if let Some(Key) = RegistryKey(&Message, &Id) {
				match ipc_server.cancelled_registry.lock() {
					Ok(mut Guard) => {
						Guard.remove(&Key);
					},

					Err(e) => {
						dev_log!("ipc", "warn: [TauriIPCServer] cancel registry poisoned ({}) - degrading on settle", e);
					},
				}

				match ipc_server.settled_registry.lock() {
					Ok(mut Guard) => {
						BoundRegistry("settled", &mut *Guard);

						if !Guard.insert(Key) {
							dev_log!("ipc", "warn: [TauriIPCServer] duplicate settle for {} - degrading", Id);
						}
					},

					Err(e) => {
						dev_log!("ipc", "warn: [TauriIPCServer] settled registry poisoned ({}) - degrading on settle", e);
					},
				}
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
