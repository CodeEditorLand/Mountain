//! `MessageCompressor::DecompressMessages`

use super::Struct;
use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct, CompressedData:&[u8]) -> Result<Vec<TauriIPCMessage>, String> {
		dev_log!("encryption", "[MessageCompressor] Decompressing {} bytes", CompressedData.len());

		let compressed_size = CompressedData.len();

		// Decompress using Gzip
		let mut decoder = GzDecoder::new(CompressedData);

		let mut DecompressedData = Vec::new();

		decoder
			.read_to_end(&mut DecompressedData)
			.map_err(|E| format!("Failed to decompress data: {}", e))?;

		let decompressed_size = DecompressedData.len();

		// Deserialize messages with explicit type annotation
		let messages:Vec<TauriIPCMessage> =
			serde_json::from_slice(&DecompressedData).map_err(|E| format!("Failed to deserialize messages: {}", e))?;

		dev_log!(
			"encryption",
			"[MessageCompressor] Decompression complete: {} -> {} bytes, {} messages",
			compressed_size,
			decompressed_size,
			messages.len()
		);

		Ok(messages)
	}
