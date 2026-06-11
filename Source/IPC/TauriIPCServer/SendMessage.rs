//! Sends a Message to the Wind frontend, queueing it for later
//! delivery when the connection is down. Body of
//! `TauriIPCServer::send`.

use super::{TauriIPCMessage, TauriIPCServer};
use crate::dev_log;

pub(crate) async fn Fn(Server:&TauriIPCServer, channel:&str, data:serde_json::Value) -> Result<(), String> {
	let Message = TauriIPCMessage {
		channel:channel.to_string(),

		data,

		sender:Some("mountain".to_string()),

		timestamp:std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64,
	};

	let is_connected = {
		let guard = Server
			.is_connected
			.lock()
			.map_err(|e| format!("Failed to check connection status: {}", e))?;

		*guard
	};

	if !is_connected {
		// Queue the Message for later delivery
		let mut queue = Server
			.message_queue
			.lock()
			.map_err(|e| format!("Failed to access Message queue: {}", e))?;

		queue.push(Message);

		dev_log!(
			"ipc",
			"[TauriIPCServer] Message queued (channel: {}, queue size: {})",
			channel,
			queue.len()
		);

		return Ok(());
	}

	// Send immediately
	Server.emit_message(&Message).await
}
