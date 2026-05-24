//! `Struct::ShouldFlush`

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

pub fn Fn(This:&Struct) -> bool {
	if This.CurrentBatch.is_empty() {
		return false;
	}

	if This.CurrentBatch.len() >= This.Config.MaxBatchSize {
		return true;
	}

	if let Some(start_time) = This.BatchStartTime {
		let elapsed = start_time.elapsed();

		if elapsed.as_millis() >= This.Config.MaxBatchDelayMs as u128 {
			return true;
		}
	}

	false
}
