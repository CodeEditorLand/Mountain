//! `Struct::AddMessage`

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

pub fn Fn(This:&mut Struct, MessageData:&[u8]) -> bool {
	let MessageSize = MessageData.len();

	let _should_compress = MessageSize >= This.Config.CompressionThresholdBytes;

	if This.BatchSizeBytes + MessageSize > This.Config.MaxBatchSize * 1024 {
		return false;
	}

	This.CurrentBatch.push_back(MessageData.to_vec());

	This.BatchSizeBytes += MessageSize;

	if This.BatchStartTime.is_none() {
		This.BatchStartTime = Some(Instant::now());
	}

	true
}
