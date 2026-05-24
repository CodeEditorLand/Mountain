//! `Compress::ShouldCompressSingle`

use super::Struct;
use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Serialize;
use super::super::Define::DefineMessage::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct, Message:&TauriIPCMessage) -> Result<bool, String> {
		// Serialize to JSON to check size
		let serialized = serde_json::to_vec(Message).map_err(|E| format!("Failed to serialize message: {}", e))?;

		Ok(serialized.len() >= This.SingleMessageThreshold)
	}
