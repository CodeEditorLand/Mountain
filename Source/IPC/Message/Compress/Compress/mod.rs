//!
//! ## Dependents
//!
//! - `TauriIPCServer` - Uses compression for message batches
//! - `Send` - Compresses outgoing messages
//! - `Receive` - Decompresses incoming messages
//!
//! ## VSCode Pattern Reference
//!
//! Inspired by VSCode's RPC message compression in
//! `vs/base/parts/ipc/node/ipc.net.ts`
//! - Adaptive compression based on payload size
//! - Size threshold for deciding when to compress
//! - Batching small messages for efficiency
//!
//! ## Security Considerations
//!
//! - Compression bomb protection: Maximum decompressed size enforced
//! - Validation of decompressed size before processing
//! - Size limits on both compression and decompression operations
//! - Fallback to uncompressed on failure prevents DoS via corruption
//!
//! ## Performance Considerations
//!
//! - Compression performed synchronously (simpler, small payloads)
//! - Compress only messages > 1KB to avoid overhead on small data
//! - Adaptive compression level (default = 6, balanced speed/ratio)
//! - Batch multiple small messages when ShouldBatch returns true
//!
//! ## Error Handling Strategy
//!
//! - Returns Result<T, CompressionError> for explicit error handling
//! - Logs compression failures without propagating to caller (graceful
//!   degradation)
//! - Falls back to uncompressed on compression failure
//! - Detailed error messages include context for debugging
//!
//! ## Thread Safety
//!
//! - All methods are `&self` and safe for concurrent access
//! - No interior mutability, state is configuration only
//!
//! ## TODO Items
//!
//! - [ ] Add support for alternative compression algorithms (zstd, brotli)
//! - [ ] Implement adaptive compression based on message type
//! - [ ] Add compression statistics tracking
pub mod New;
pub mod Defaults;
pub mod CompressMessages;
pub mod DecompressMessages;
pub mod ShouldBatch;
pub mod ShouldCompressSingle;

use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Serialize;
use super::super::Define::DefineMessage::TauriIPCMessage;
use crate::dev_log;

type being compressed

/// Message compression utility for optimizing IPC message transfer
/// This struct provides Gzip-based compression for IPC messages, adapting the
/// compression level based on payload size and providing graceful fallback on
/// errors.
pub struct Struct {
	/// Compression level (0-9), higher = better ratio but slower
	/// 0 = no compression, 1 = fastest, 6 = balanced, 9 = best
	CompressionLevel:u32,

	/// Minimum number of messages required for batching
	BatchSize:usize,

	/// Minimum size in bytes before compressing a single message
	SingleMessageThreshold:usize,
}
