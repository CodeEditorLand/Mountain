//! `Server::Initialize`

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
	dev_log!("ipc", "[TauriIPCServer] Setting up IPC listeners");

	// Set up connection status
	{
		let mut is_connected = self
			.IsConnected
			.lock()
			.map_err(|E| format!("Failed to lock connection status: {}", e))?;

		*is_connected = true;
	}

	// Initialize permission manager
	{
		let mut permission_manager = self
			.permission_manager
			.lock()
			.map_err(|E| format!("Failed to lock permission manager: {}", e))?;

		if permission_manager.is_none() {
			let pm = PermissionManager::new();

			let pm_clone = pm.clone();

			tokio::spawn(async move {
				pm_clone.InitializeDefaults().await;
			});

			*permission_manager = Some(pm);
		}
	}

	// Notify Wind that Mountain is ready
	This.send_connection_status(true)
		.await
		.map_err(|E| format!("Failed to send connection status: {}", e))?;

	dev_log!("ipc", "[TauriIPCServer] IPC Server initialized successfully");

	// Process any queued messages
	This.process_message_queue().await;

	Ok(())
}
