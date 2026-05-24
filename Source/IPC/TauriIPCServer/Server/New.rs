//! `Server::New`

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

pub fn Fn(app_handle:AppHandle) -> Struct {
	dev_log!("ipc", "[TauriIPCServer] Initializing Mountain IPC Server");

	Self {
		app_handle,

		listeners:Arc::new(Mutex::new(HashMap::new())),

		is_connected:Arc::new(Mutex::new(false)),

		message_queue:Arc::new(Mutex::new(Vec::new())),

		permission_manager:Arc::new(Mutex::new(None)),
	}
}
