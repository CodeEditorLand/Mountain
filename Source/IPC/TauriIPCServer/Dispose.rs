//! Clears listeners, queued messages, and connection state. Body of
//! `TauriIPCServer::dispose`.

use super::TauriIPCServer;
use crate::dev_log;

pub(crate) fn Fn(Server:&TauriIPCServer) -> Result<(), String> {
	{
		let mut listeners = Server
			.listeners
			.lock()
			.map_err(|e| format!("Failed to access listeners: {}", e))?;

		listeners.clear();
	}

	{
		let mut queue = Server
			.message_queue
			.lock()
			.map_err(|e| format!("Failed to access Message queue: {}", e))?;

		queue.clear();
	}

	{
		let mut is_connected = Server
			.is_connected
			.lock()
			.map_err(|e| format!("Failed to access connection status: {}", e))?;

		*is_connected = false;
	}

	dev_log!("ipc", "[TauriIPCServer] IPC Server disposed");

	Ok(())
}
