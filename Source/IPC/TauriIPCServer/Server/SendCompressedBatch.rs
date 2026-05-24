//! `Server::SendCompressedBatch`

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

pub fn Fn(&self, channel:&str, messages:Vec<TauriIPCMessage>) -> Result<(), String> {
	// Configure compressor with balanced settings
	let compressor = MessageCompressor::new(6, 10);

	let compressed_data = compressor
		.CompressMessages(messages)
		.map_err(|E| format!("Failed to compress batch: {}", e))?;

	let batch_message = TauriIPCMessage {
		channel:"compressed_batch".to_string(),

		data:serde_json::Value::String(base64::encode(&compressed_data)),

		sender:Some("mountain".to_string()),

		timestamp:std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64,
	};

	This.send(channel, serde_json::to_value(batch_message).unwrap()).await
}
