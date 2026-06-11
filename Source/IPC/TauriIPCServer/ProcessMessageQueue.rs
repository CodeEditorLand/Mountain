//! Drains queued messages to Wind, re-queueing on emit failure. Body
//! of `TauriIPCServer::process_message_queue`.

use super::TauriIPCServer;
use crate::dev_log;

pub(crate) async fn Fn(Server:&TauriIPCServer) {
	let mut queue = match Server.message_queue.lock() {
		Ok(queue) => queue,

		Err(e) => {
			dev_log!("ipc", "error: [TauriIPCServer] Failed to access Message queue: {}", e);

			return;
		},
	};

	while let Some(Message) = queue.pop() {
		if let Err(e) = Server.emit_message(&Message).await {
			dev_log!("ipc", "error: [TauriIPCServer] Failed to send queued Message: {}", e);

			// Put the Message back in the queue
			queue.insert(0, Message);

			break;
		}
	}

	dev_log!(
		"ipc",
		"[TauriIPCServer] Message queue processed, {} messages remaining",
		queue.len()
	);
}
