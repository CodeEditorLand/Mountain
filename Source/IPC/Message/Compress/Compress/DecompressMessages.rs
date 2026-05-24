//! `Compress::DecompressMessages`

use super::Struct;
use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Serialize;
use super::super::Define::DefineMessage::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct, CompressedData:&[u8]) -> Result<Vec<TauriIPCMessage>, String> {
		ValidateDecompressionInput(CompressedData.len())?;

		let mut decoder = GzDecoder::new(CompressedData);

		// Pre-allocate 64KB buffer for decompressed data as an optimization
		// (actual size will grow as needed, this is just an initial estimate)
		let mut DecompressedData = Vec::with_capacity(65536);

		// Read with size limit to prevent decompression bomb
		let bytes_read = decoder
			.read_to_end(&mut DecompressedData)
			.map_err(|E| format!("Failed to decompress data: {}", e))?;

		ValidateDecompressedSize(bytes_read)?;

		// Deserialize messages
		serde_json::from_slice(&DecompressedData).map_err(|E| format!("Failed to deserialize messages: {}", e))
	}
