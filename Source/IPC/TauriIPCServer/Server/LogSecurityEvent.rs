//! `Server::LogSecurityEvent`

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

pub fn Fn(This:&Struct, event:SecurityEvent) {
	let permission_manager_guard = match This.permission_manager.lock() {
		Ok(guard) => guard,

		Err(e) => {
			dev_log!("ipc", "error: [TauriIPCServer] Failed to access permission manager: {}", e);

			return;
		},
	};

	if let Some(permission_manager) = permission_manager_guard.as_ref() {
		permission_manager.LogSecurityEvent(event).await;
	}
}
