//! `Struct::GetBatchStats`

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

pub fn Fn(This:&Struct) -> BatchStats {
	BatchStats {
		messages_count:This.CurrentBatch.len(),

		total_size_bytes:This.BatchSizeBytes,

		batch_age_ms:This.BatchStartTime.map(|t| t.elapsed().as_millis() as u64).unwrap_or(0),
	}
}
