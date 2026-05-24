//! `Server::On`

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

pub fn Fn(This:&Struct, channel:&str, callback:ListenerCallback) -> Result<(), String> {
	let mut listeners = self
		.listeners
		.lock()
		.map_err(|E| format!("Failed to access listeners: {}", e))?;

	listeners.entry(channel.to_string()).or_insert_with(Vec::new).push(callback);

	dev_log!("ipc", "[TauriIPCServer] Listener registered for channel: {}", channel);

	Ok(())
}
