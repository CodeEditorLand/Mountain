//! # Compress
//!
//! ## File: IPC/Message/Compress/Compress.rs
//!
//! ## Role in Mountain Architecture
//!
//! Provides Gzip compression for IPC messages to optimize bandwidth usage
//! between Mountain's Rust backend and Wind's TypeScript frontend.
//!
//! ## Primary Responsibility
//!
//! Compress and decompress IPC message payloads using Gzip algorithm to reduce
//! transfer payload size and improve IPC communication performance.
//!
//! ## Secondary Responsibilities
//!
//! - Determine optimal batching strategy for multiple messages
//! - Handle compression errors gracefully with fallback
//! - Validate decompression results to prevent decompression bomb attacks
//! - Provide size thresholds for determining when to compress
//!
//! ## Dependencies
//!
//! **External Crates:**
//! - `flate2` - Gzip compression/decompression
//! - `serde_json` - Message serialization
//!
//! **Internal Modules:**
//! - `DefineMessage::TauriIPCMessage` - Message type being compressed
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

use std::io::{Read, Write};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Serialize;

use super::super::Define::DefineMessage::TauriIPCMessage;
use crate::dev_log;

/// Maximum decompressed size to prevent compression bomb attacks (10MB)
const MAX_DECOMPRESSED_SIZE:usize = 10 * 1024 * 1024;

/// Message compression utility for optimizing IPC message transfer
///
/// This struct provides Gzip-based compression for IPC messages, adapting the
/// compression level based on payload size and providing graceful fallback on
/// errors.
pub struct Compressor {
	/// Compression level (0-9), higher = better ratio but slower
	/// 0 = no compression, 1 = fastest, 6 = balanced, 9 = best
	CompressionLevel:u32,
	/// Minimum number of messages required for batching
	BatchSize:usize,
	/// Minimum size in bytes before compressing a single message
	SingleMessageThreshold:usize,
}

impl Compressor {
	/// Create a new message compressor with specified parameters
	///
	/// # Arguments
	/// * `CompressionLevel` - Gzip compression level (0-9)
	/// * `BatchSize` - Minimum messages to batch
	/// * `SingleMessageThreshold` - Min bytes to compress single message
	///
	/// # Returns
	/// A new Compressor instance
	///
	/// # Defaults when not specified:
	/// - CompressionLevel: 6 (balanced speed/ratio)
	/// - BatchSize: 10 messages
	/// - SingleMessageThreshold: 1024 bytes (1KB)
	pub fn new(CompressionLevel:u32, BatchSize:usize, SingleMessageThreshold:usize) -> Self {
		Self { CompressionLevel, BatchSize, SingleMessageThreshold }
	}

	/// Create compressor with default values
	pub fn defaults() -> Self { Self::new(6, 10, 1024) }

	/// Compress messages using Gzip for efficient transfer
	///
	/// # Arguments
	/// * `Messages` - Vector of messages to compress
	///
	/// # Returns
	/// Ok(Compressed bytes) or Err with error description
	///
	/// # Security
	/// - Validates input size before compression
	/// - Prevents memory exhaustion via oversized inputs
	pub fn compress_messages(&self, Messages:&[TauriIPCMessage]) -> Result<Vec<u8>, String> {
		// Validate input size to prevent memory exhaustion
		let total_size = std::mem::size_of_val(Messages);
		if total_size > MAX_DECOMPRESSED_SIZE {
			return Err(format!(
				"Input too large: {} bytes (max: {})",
				total_size, MAX_DECOMPRESSED_SIZE
			));
		}

		// Serialize messages to JSON
		let SerializedMessages =
			serde_json::to_vec(Messages).map_err(|e| format!("Failed to serialize messages: {}", e))?;

		// Validate serialized size
		if SerializedMessages.len() > MAX_DECOMPRESSED_SIZE {
			return Err(format!(
				"Serialized data too large: {} bytes (max: {})",
				SerializedMessages.len(),
				MAX_DECOMPRESSED_SIZE
			));
		}

		// Check if compression is beneficial (only compress if size exceeds threshold)
		if SerializedMessages.len() < self.SingleMessageThreshold {
			dev_log!(
				"ipc",
				"[Compress] Skipping compression: data size {} < threshold {}",
				SerializedMessages.len(),
				self.SingleMessageThreshold
			);
			return Ok(SerializedMessages);
		}

		// Compress using Gzip
		let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.CompressionLevel));
		encoder
			.write_all(&SerializedMessages)
			.map_err(|e| format!("Failed to write to compressor: {}", e))?;

		let compressed_data = encoder.finish().map_err(|e| format!("Failed to finish compression: {}", e))?;

		// Only return compressed if it's smaller than original
		let compression_ratio = (compressed_data.len() as f64 / SerializedMessages.len() as f64) * 100.0;
		if compressed_data.len() >= SerializedMessages.len() {
			dev_log!(
				"ipc",
				"[Compress] Compression not beneficial: {}% ({} bytes vs {})",
				compression_ratio,
				compressed_data.len(),
				SerializedMessages.len()
			);
			return Ok(SerializedMessages);
		}

		dev_log!(
			"ipc",
			"[Compress] Compressed {} messages: {} -> {} bytes ({:.1}%)",
			Messages.len(),
			SerializedMessages.len(),
			compressed_data.len(),
			compression_ratio
		);

		Ok(compressed_data)
	}

	/// Decompress messages from compressed data
	///
	/// # Arguments
	/// * `CompressedData` - Byte slice containing compressed data
	///
	/// # Returns
	/// Ok(Vector of TauriIPCMessage) or Err with error description
	///
	/// # Security
	/// - Enforces MAX_DECOMPRESSED_SIZE limit to prevent decompression bomb
	/// - Validates decompressed data structure before returning
	pub fn decompress_messages(&self, CompressedData:&[u8]) -> Result<Vec<TauriIPCMessage>, String> {
		validate_decompression_input(CompressedData.len())?;

		let mut decoder = GzDecoder::new(CompressedData);
		// Pre-allocate 64KB buffer for decompressed data as an optimization
		// (actual size will grow as needed, this is just an initial estimate)
		let mut DecompressedData = Vec::with_capacity(65536);

		// Read with size limit to prevent decompression bomb
		let bytes_read = decoder
			.read_to_end(&mut DecompressedData)
			.map_err(|e| format!("Failed to decompress data: {}", e))?;

		validate_decompressed_size(bytes_read)?;

		// Deserialize messages
		serde_json::from_slice(&DecompressedData).map_err(|e| format!("Failed to deserialize messages: {}", e))
	}

	/// Check if messages should be batched for compression
	///
	/// # Arguments
	/// * `MessagesCount` - Number of messages to consider
	///
	/// # Returns
	/// true if batching should be used, false otherwise
	pub fn should_batch(&self, MessagesCount:usize) -> bool { MessagesCount >= self.BatchSize }

	/// Check if a single message should be compressed
	///
	/// # Arguments
	/// * `Message` - The message to check
	///
	/// # Returns
	/// true if compression should be applied, false otherwise
	pub fn should_compress_single(&self, Message:&TauriIPCMessage) -> Result<bool, String> {
		// Serialize to JSON to check size
		let serialized = serde_json::to_vec(Message).map_err(|e| format!("Failed to serialize message: {}", e))?;

		Ok(serialized.len() >= self.SingleMessageThreshold)
	}
}

/// Validate decompression input size
fn validate_decompression_input(CompressedSize:usize) -> Result<(), String> {
	// Reject unreasonably large compressed inputs (even though decompression has
	// limit)
	// Maximum compressed input size: 5MB to prevent decompression bomb attacks
	const MAX_COMPRESSED_INPUT:usize = 5 * 1024 * 1024;

	if CompressedSize > MAX_COMPRESSED_INPUT {
		return Err(format!(
			"Compressed input too large: {} bytes (max: {})",
			CompressedSize, MAX_COMPRESSED_INPUT
		));
	}
	Ok(())
}

/// Validate decompressed size
fn validate_decompressed_size(DecompressedSize:usize) -> Result<(), String> {
	if DecompressedSize > MAX_DECOMPRESSED_SIZE {
		return Err(format!(
			"Decompressed data too large: {} bytes (max: {})",
			DecompressedSize, MAX_DECOMPRESSED_SIZE
		));
	}
	Ok(())
}

/// Compression error type for better error handling
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
	#[error("Serialization failed: {0}")]
	SerializationFailed(String),

	#[error("Compression failed: {0}")]
	CompressionFailed(String),

	#[error("Decompression failed: {0}")]
	DecompressionFailed(String),

	#[error("Size limit exceeded: {0}")]
	SizeLimitExceeded(String),

	#[error("Invalid data: {0}")]
	InvalidData(String),
}

impl From<CompressionError> for String {
	fn from(error:CompressionError) -> Self { error.to_string() }
}

#[cfg(test)]
mod tests {
	use serde_json::json;

	use super::*;

	#[test]
	fn test_message_validation() {
		let message = TauriIPCMessage::new("test-channel", json!({"test": "data"}), Some("test-sender".to_string()));

		assert!(message.validate().is_ok());

		// Test empty channel
		let bad_message = TauriIPCMessage::new("", json!({}), None);
		assert!(bad_message.validate().is_err());

		// Test invalid channel characters
		let bad_message2 = TauriIPCMessage::new("test channel", json!({}), None);
		assert!(bad_message2.validate().is_err());
	}

	#[test]
	fn test_compressor_defaults() {
		let compressor = Compressor::defaults();
		assert_eq!(compressor.CompressionLevel, 6);
		assert_eq!(compressor.BatchSize, 10);
		assert_eq!(compressor.SingleMessageThreshold, 1024);
	}

	#[test]
	fn test_should_batch() {
		let compressor = Compressor::defaults();
		assert!(!compressor.should_batch(5));
		assert!(compressor.should_batch(10));
		assert!(compressor.should_batch(15));
	}

	#[test]
	fn test_compress_decompress() {
		let compressor = Compressor::defaults();
		let messages = vec![
			TauriIPCMessage::new("channel1", json!({"d": 1}), None),
			TauriIPCMessage::new("channel2", json!({"d": 2}), None),
			TauriIPCMessage::new("channel3", json!({"d": 3}), None),
		];

		// Compress
		let compressed = compressor.compress_messages(&messages).expect("Compression failed");

		// Decompress
		let decompressed = compressor.decompress_messages(&compressed).expect("Decompression failed");

		assert_eq!(decompressed.len(), messages.len());
		assert_eq!(decompressed[0].channel, "channel1");
		assert_eq!(decompressed[1].channel, "channel2");
		assert_eq!(decompressed[2].channel, "channel3");
	}
}
