//! `Server::CompressedBatch`

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
	let compressed_data_base64 = Message.data.as_str().ok_or("Compressed batch data must be a string")?;

	let compressed_data =
		base64::decode(compressed_data_base64).map_err(|E| format!("Failed to decode base64: {}", e))?;

	let compressor = MessageCompressor::new(6, 10);

	let messages = compressor
		.DecompressMessages(&compressed_data)
		.map_err(|E| format!("Failed to decompress batch: {}", e))?;

	// Process each Message in the batch
	for msg in messages {
		This.IncomingMessage(msg).await?;
	}

	Ok(())
}
