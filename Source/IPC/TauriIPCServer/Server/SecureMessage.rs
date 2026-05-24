//! `Server::SecureMessage`

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

pub fn Fn(This:&Struct, encrypted_data:serde_json::Value) -> Result<(), String> {
	use serde::Deserialize;

	#[derive(Deserialize)]
	struct EncryptedMessage {
		nonce:Vec<u8>,

		ciphertext:Vec<u8>,

		hmac_tag:Vec<u8>,
	}

	let encrypted_message:EncryptedMessage = serde_json::from_value(encrypted_data)
		.map_err(|E| format!("Failed to deserialize encrypted Message: {}", e))?;

	let secure_channel = SecureMessageChannel::new().map_err(|E| format!("Failed to create secure channel: {}", e))?;

	let Message = secure_channel
		.DecryptMessage(&super::super::Encryption::EncryptedMessage {
			nonce:encrypted_message.nonce,
			ciphertext:encrypted_message.ciphertext,
			hmac_tag:encrypted_message.hmac_tag,
		})
		.map_err(|E| format!("Failed to decrypt Message: {}", e))?;

	This.IncomingMessage(Message).await
}
