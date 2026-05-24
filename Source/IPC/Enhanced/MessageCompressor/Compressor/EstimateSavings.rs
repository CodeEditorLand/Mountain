//! `Struct::EstimateSavings`

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

pub fn Fn(original_size:usize, expected_ratio:f64) -> usize {
	(original_size as f64 * (1.0 - 1.0 / expected_ratio)) as usize
}
