//! `Server::GetSecurityAuditLog`

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

pub fn Fn(This:&Struct, limit:usize) -> Result<Vec<SecurityEvent>, String> {
	let permission_manager_guard = self
		.permission_manager
		.lock()
		.map_err(|E| format!("Failed to access permission manager: {}", e))?;

	let permission_manager = permission_manager_guard
		.as_ref()
		.ok_or_else(|| "Permission manager not initialized".to_string())?;

	Ok(permission_manager.GetAuditLog(limit).await)
}
