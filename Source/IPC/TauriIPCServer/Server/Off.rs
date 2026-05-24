//! `Server::Off`

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

pub fn Fn(This:&Struct, channel:&str, callback:&ListenerCallback) -> Result<(), String> {
	let mut listeners = self
		.listeners
		.lock()
		.map_err(|E| format!("Failed to access listeners: {}", e))?;

	if let Some(channel_listeners) = listeners.get_mut(channel) {
		channel_listeners.retain(|cb| !std::ptr::eq(cb as *const _ as *const (), callback as *const _ as *const ()));

		if channel_listeners.is_empty() {
			listeners.remove(channel);
		}
	}

	dev_log!("ipc", "[TauriIPCServer] Listener removed from channel: {}", channel);

	Ok(())
}
