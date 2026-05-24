//! `Server::IncomingMessage`

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

pub fn Fn(This:&Struct, Message:TauriIPCMessage) -> Result<(), String> {
	dev_log!("ipc", "[TauriIPCServer] Received Message on channel: {}", Message.channel);

	let listeners = self
		.listeners
		.lock()
		.map_err(|E| format!("Failed to access listeners: {}", e))?;

	if let Some(channel_listeners) = listeners.get(&Message.channel) {
		for callback in channel_listeners {
			if let Err(e) = callback(Message.data.clone()) {
				dev_log!(
					"ipc",
					"error: [TauriIPCServer] Error in listener for channel {}: {}",
					Message.channel,
					e
				);
			}
		}
	} else {
		dev_log!("ipc", "[TauriIPCServer] No listeners found for channel: {}", Message.channel);
	}

	Ok(())
}
