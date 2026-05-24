//! `Server::Send`

use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

use tauri::{AppHandle, Emitter, Manager};

use super::{
	super::{
		Connection::{ConnectionManager, ConnectionStats},
		Encryption::{SecureMessageChannel, Struct},
		Message::{ConnectionStatus, ListenerCallback, TauriIPCMessage},
		Security::PermissionManager::{
			Manager::Struct as PermissionManager,
			SecurityContext::Struct as SecurityContext,
			SecurityEvent::Struct as SecurityEvent,
			SecurityEventType::Enum as SecurityEventType,
		},
	},
	Struct,
};
use crate::dev_log;

pub fn Fn(This:&Struct, channel:&str, data:serde_json::Value) -> Result<(), String> {
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
		let guard = self
			.IsConnected
			.lock()
			.map_err(|E| format!("Failed to check connection status: {}", e))?;

		*guard
	};

	if !is_connected {
		// Queue the Message for later delivery
		let mut queue = self
			.message_queue
			.lock()
			.map_err(|E| format!("Failed to access Message queue: {}", e))?;

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
	This.emit_message(&Message).await
}
