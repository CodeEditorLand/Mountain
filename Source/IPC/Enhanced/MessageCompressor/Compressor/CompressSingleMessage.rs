//! `Struct::CompressSingleMessage`

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

pub fn Fn(
	message_data:&[u8],

	algorithm:CompressionAlgorithm,

	level:CompressionLevel,
) -> Result<(Vec<u8>, CompressionInfo), String> {
	let config = BatchConfig { Algorithm:algorithm, CompressionLevel:level, ..Default::default() };

	let compressor = Struct::new(config);

	compressor.compress_data(message_data)
}
