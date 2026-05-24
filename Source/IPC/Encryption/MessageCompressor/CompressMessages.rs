//! `MessageCompressor::CompressMessages`

use super::Struct;
use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct, Messages:Vec<TauriIPCMessage>) -> Result<Vec<u8>, String> {
		dev_log!("encryption", "[MessageCompressor] Compressing {} messages", Messages.len());

		// Serialize messages to JSON
		let SerializedMessages =
			serde_json::to_vec(&Messages).map_err(|E| format!("Failed to serialize messages: {}", e))?;

		let original_size = SerializedMessages.len();

		// Compress using Gzip
		let mut encoder = GzEncoder::new(Vec::new(), Compression::new(This.CompressionLevel));

		encoder
			.write_all(&SerializedMessages)
			.map_err(|E| format!("Failed to compress messages: {}", e))?;

		let compressed_data = encoder.finish().map_err(|E| format!("Failed to finish compression: {}", e))?;

		let compressed_size = compressed_data.len();

		let ratio = if original_size > 0 {
			(compressed_size as f64 / original_size as f64) * 100.0
		} else {
			100.0
		};

		dev_log!(
			"encryption",
			"[MessageCompressor] Compression complete: {} -> {} bytes ({:.1}%)",
			original_size,
			compressed_size,
			ratio
		);

		Ok(compressed_data)
	}
