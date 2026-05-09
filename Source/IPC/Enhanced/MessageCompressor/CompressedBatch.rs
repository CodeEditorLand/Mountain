#![allow(non_snake_case)]

//! Result envelope returned by `Compressor::Struct::flush_batch`.
//! Carries the message count, original / compressed byte
//! totals, the compressed bytes (`None` when below threshold),
//! the `CompressionInfo::Struct`, and the flush timestamp.

use serde::{Deserialize, Serialize};

use crate::IPC::Enhanced::MessageCompressor::CompressionInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Struct {
	pub messages_count:usize,

	pub original_size:usize,

	pub compressed_size:usize,

	pub compressed_data:Option<Vec<u8>>,

	pub compression_info:CompressionInfo::Struct,

	pub timestamp:u64,
}
