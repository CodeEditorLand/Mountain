pub mod New;
pub mod CompressMessages;
pub mod DecompressMessages;
pub mod ShouldBatch;
pub mod CompressionLevel;
pub mod BatchSize;
pub mod Default;
pub mod Fast;
pub mod Max;

use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

/// Message compression utility for optimizing IPC message transfer
/// This structure provides Gzip-based compression to reduce the size of IPC
/// messages, improving transfer speed and reducing bandwidth usage.
/// ## Compression Flow
/// ```text
/// Multiple TauriIPCMessage
///     |
///     | 1. Serialize to JSON
///     v
/// Serialized JSON bytes
///     |
///     | 2. Compress with Gzip
///     v
/// Compressed bytes (smaller)
///     |
///     | 3. Base64 encode for transport
///     v
/// Base64 string (sendable via IPC)
/// ```
/// ## Decompression Flow
/// ```text
/// Base64 string (received via IPC)
///     |
///     | 1. Base64 decode
///     v
/// Compressed bytes
///     |
///     | 2. Decompress with Gzip
///     v
/// Serialized JSON bytes
///     |
///     | 3. Deserialize to TauriIPCMessage[]
///     v
/// Multiple TauriIPCMessage
/// ```
/// ## Compression Levels
/// Compression levels range from 0 (fastest, no compression) to 9 (slowest,
/// best compression). The recommended level is 6 for a good balance.
/// | Level | Speed | Ratio | Use Case |
/// |-------|-------|-------|----------|
/// | 0 | Fastest | 1:1 | Testing/debugging |
/// | 1-3 | Fast | 2:1-3:1 | Real-time systems |
/// | 4-6 | Medium | 3:1-5:1 | General use |
/// | 7-9 | Slow | 5:1-7:1 | Bandwidth-constrained |
/// ## Example Usage
/// ```rust,ignore
/// let compressor = MessageCompressor::new(6, 10);
/// // Compress messages
/// let messages = vec![message1, message2, message3];
/// let compressed = compressor.CompressMessages(messages)?;
/// // Decompress messages
/// let decompressed = compressor.DecompressMessages(&compressed)?;
/// ```
pub struct Struct {
	/// Gzip compression level (0-9, where 0 is no compression)
	CompressionLevel:u32,

	/// Minimum number of messages required for batch processing
	BatchSize:usize,
}
