//! `Struct::FlushBatch`

use std::{
	collections::VecDeque,
	io::{Read, Write},
};

use bincode::serde::{decode_from_slice, encode_to_vec};
use brotli::{CompressorReader, CompressorWriter, enc::BrotliEncoderParams};
use flate2::{
	Compression,
	write::{GzEncoder, ZlibEncoder},
};
use tokio::time::Instant;

use super::Struct;
use crate::IPC::Enhanced::MessageCompressor::{
	BatchConfig::Struct as BatchConfig,
	BatchStats::Struct as BatchStats,
	CompressedBatch::Struct as CompressedBatch,
	CompressionAlgorithm::Enum as CompressionAlgorithm,
	CompressionInfo::Struct as CompressionInfo,
	CompressionLevel::Enum as CompressionLevel,
};

pub fn Fn(This:&mut Struct) -> Result<CompressedBatch, String> {
	if This.CurrentBatch.is_empty() {
		return Err("No messages in batch to flush".to_string());
	}

	let BatchMessages:Vec<Vec<u8>> = This.CurrentBatch.drain(..).collect();

	let total_size = This.BatchSizeBytes;

	This.BatchStartTime = None;

	This.BatchSizeBytes = 0;

	let config = bincode::config::standard();

	let serialized_batch =
		encode_to_vec(&BatchMessages, config).map_err(|E| format!("Failed to serialize batch: {}", e))?;

	let (CompressedData, compression_info) = if total_size >= This.Config.CompressionThresholdBytes {
		This.compress_data(&serialized_batch).map(|(data, info)| (Some(data), info))
	} else {
		Ok((None, CompressionInfo::none()))
	}?;

	Ok(CompressedBatch {
		messages_count:BatchMessages.len(),
		original_size:total_size,
		compressed_size:CompressedData.as_ref().map(|d| d.len()).unwrap_or(total_size),
		compressed_data:CompressedData,
		compression_info,
		timestamp:std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap_or_default()
			.as_millis() as u64,
	})
}
