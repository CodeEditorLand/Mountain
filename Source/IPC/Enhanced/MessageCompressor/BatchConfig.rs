#![allow(non_snake_case)]

//! Compressor / batcher tunables - max batch size, max delay
//! before flushing, the size threshold below which messages
//! pass through uncompressed, and the algorithm + level pair.

use serde::{Deserialize, Serialize};

use crate::IPC::Enhanced::MessageCompressor::{CompressionAlgorithm, CompressionLevel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub MaxBatchSize:usize,

	pub MaxBatchDelayMs:u64,

	pub CompressionThresholdBytes:usize,

	pub CompressionLevel:CompressionLevel::Enum,

	pub Algorithm:CompressionAlgorithm::Enum,
}

impl Default for Struct {
	fn default() -> Self {
		Self {
			MaxBatchSize:100,

			MaxBatchDelayMs:100,

			CompressionThresholdBytes:1024,

			CompressionLevel:CompressionLevel::Enum::Balanced,

			Algorithm:CompressionAlgorithm::Enum::Brotli,
		}
	}
}
