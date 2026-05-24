pub mod New;
pub mod AddMessage;
pub mod ShouldFlush;
pub mod FlushBatch;
pub mod DecompressBatch;
pub mod GetBatchStats;
pub mod ClearBatch;
pub mod CompressSingleMessage;
pub mod CalculateCompressionRatio;
pub mod EstimateSavings;

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

use crate::IPC::Enhanced::MessageCompressor::{
	BatchConfig::Struct as BatchConfig,
	BatchStats::Struct as BatchStats,
	CompressedBatch::Struct as CompressedBatch,
	CompressionAlgorithm::Enum as CompressionAlgorithm,
	CompressionInfo::Struct as CompressionInfo,
	CompressionLevel::Enum as CompressionLevel,
};

pub struct Struct {
	pub(super) Config:BatchConfig,

	pub(super) CurrentBatch:VecDeque<Vec<u8>>,

	pub(super) BatchStartTime:Option<Instant>,

	pub(super) BatchSizeBytes:usize,
}
