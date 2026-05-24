//! `Server::SendWithPool`

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
	let pool = Arc::new(ConnectionManager::new(10, std::time::Duration::from_secs(30)));

	let Handle = pool
		.GetConnection()
		.await
		.map_err(|E| format!("Failed to get connection: {}", e))?;

	let result = This.send(channel, data).await;

	pool.ReleaseConnection(Handle).await;

	result
}
