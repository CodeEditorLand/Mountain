//! `Server::SendSecure`

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
	let secure_channel = SecureMessageChannel::new().map_err(|E| format!("Failed to create secure channel: {}", e))?;

	let Message = TauriIPCMessage {
		channel:channel.to_string(),

		data,

		sender:Some("mountain".to_string()),

		timestamp:std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64,
	};

	let encrypted_message = secure_channel
		.EncryptMessage(&Message)
		.map_err(|E| format!("Failed to encrypt Message: {}", e))?;

	let encrypted_data =
		serde_json::to_value(encrypted_message).map_err(|E| format!("Failed to serialize encrypted Message: {}", e))?;

	This.send("secure_message", encrypted_data).await
}
