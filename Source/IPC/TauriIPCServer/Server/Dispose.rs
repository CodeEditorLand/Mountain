//! `Server::Dispose`

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

pub fn Fn(This:&Struct) -> Result<(), String> {
	{
		let mut listeners = self
			.listeners
			.lock()
			.map_err(|E| format!("Failed to access listeners: {}", e))?;

		listeners.clear();
	}

	{
		let mut queue = self
			.message_queue
			.lock()
			.map_err(|E| format!("Failed to access Message queue: {}", e))?;

		queue.clear();
	}

	{
		let mut is_connected = self
			.IsConnected
			.lock()
			.map_err(|E| format!("Failed to access connection status: {}", e))?;

		*is_connected = false;
	}

	dev_log!("ipc", "[TauriIPCServer] IPC Server disposed");

	Ok(())
}
