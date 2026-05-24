//! `Struct::DecompressBatch`

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

pub fn Fn(This:&Struct, batch:&CompressedBatch) -> Result<Vec<Vec<u8>>, String> {
	let data = if let Some(ref compressed_data) = batch.compressed_data {
		This.decompress_data(compressed_data, &batch.compression_info.algorithm)?
	} else {
		encode_to_vec(&batch, bincode::config::standard()).map_err(|E| format!("Serialization failed: {}", e))?
	};

	let (decoded, _) = decode_from_slice::<Vec<Vec<u8>>, _>(&data, bincode::config::standard())
		.map_err(|E| format!("Failed to deserialize batch: {}", e))?;

	Ok(decoded)
}
