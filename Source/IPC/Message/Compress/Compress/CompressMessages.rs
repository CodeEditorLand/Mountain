//! `Compress::CompressMessages`

use super::Struct;
use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Serialize;
use super::super::Define::DefineMessage::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct, Messages:&[TauriIPCMessage]) -> Result<Vec<u8>, String> {
		// Validate input size to prevent memory exhaustion
		let total_size = std::mem::size_of_val(Messages);

		if total_size > MAX_DECOMPRESSED_SIZE {
			return Err(format!(
				"Input too large: {} bytes (max: {})",
				total_size, MAX_DECOMPRESSED_SIZE
			));
		}

		// Serialize messages to JSON
		let SerializedMessages =
			serde_json::to_vec(Messages).map_err(|E| format!("Failed to serialize messages: {}", e))?;

		// Validate serialized size
		if SerializedMessages.len() > MAX_DECOMPRESSED_SIZE {
			return Err(format!(
				"Serialized data too large: {} bytes (max: {})",
				SerializedMessages.len(),
				MAX_DECOMPRESSED_SIZE
			));
		}

		// Check if compression is beneficial (only compress if size exceeds threshold)
		if SerializedMessages.len() < This.SingleMessageThreshold {
			dev_log!(
				"ipc",
				"[Compress] Skipping compression: data size {} < threshold {}",
				SerializedMessages.len(),
				This.SingleMessageThreshold
			);

			return Ok(SerializedMessages);
		}

		// Compress using Gzip
		let mut encoder = GzEncoder::new(Vec::new(), Compression::new(This.CompressionLevel));

		encoder
			.write_all(&SerializedMessages)
			.map_err(|E| format!("Failed to write to compressor: {}", e))?;

		let compressed_data = encoder.finish().map_err(|E| format!("Failed to finish compression: {}", e))?;

		// Only return compressed if it's smaller than original
		let compression_ratio = (compressed_data.len() as f64 / SerializedMessages.len() as f64) * 100.0;

		if compressed_data.len() >= SerializedMessages.len() {
			dev_log!(
				"ipc",
				"[Compress] Compression not beneficial: {}% ({} bytes vs {})",
				compression_ratio,
				compressed_data.len(),
				SerializedMessages.len()
			);

			return Ok(SerializedMessages);
		}

		dev_log!(
			"ipc",
			"[Compress] Compressed {} messages: {} -> {} bytes ({:.1}%)",
			Messages.len(),
			SerializedMessages.len(),
			compressed_data.len(),
			compression_ratio
		);

		Ok(compressed_data)
	}
