//! # Message Compressor (IPC Encryption)
//!
//! ## RESPONSIBILITIES
//! This module provides message compression using Gzip to optimize IPC message
//! transfer. It reduces payload size for better performance, especially for
//! large messages or high-frequency communication.
//!
//! ## ARCHITECTURAL ROLE
//! This module is part of the performance optimization layer in the IPC
//! architecture, reducing bandwidth usage and improving transfer speeds.
//!
//! ## KEY COMPONENTS
//!
//! - **MessageCompressor**: Main compression structure with configurable
//!   compression level
//!
//! ## ERROR HANDLING
//! Compression and decompression operations return Result types with
//! descriptive error messages for failures.
//!
//! ## LOGGING
//! Debug-level logging for compression statistics, error for failures.
//!
//! ## PERFORMANCE CONSIDERATIONS
//! - Compression level 6 provides good balance between speed and ratio
//! - Batch size 10 aggregates small messages for efficiency
//! - Gzip provides widely compatible compression format
//!
//! ## TODO
//! - Add compression algorithm selection (LZ4, Zstd)
//! - Implement adaptive compression based on message size
//! - Add compression ratio tracking and optimization
//! - Implement streaming compression for very large messages

use std::io::{Read, Write};

use flate2::{Compression, read::GzDecoder, write::GzEncoder};

use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

/// Message compression utility for optimizing IPC message transfer
///
/// This structure provides Gzip-based compression to reduce the size of IPC
/// messages, improving transfer speed and reducing bandwidth usage.
///
/// ## Compression Flow
///
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
///
/// ## Decompression Flow
///
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
///
/// ## Compression Levels
///
/// Compression levels range from 0 (fastest, no compression) to 9 (slowest,
/// best compression). The recommended level is 6 for a good balance.
///
/// | Level | Speed | Ratio | Use Case |
/// |-------|-------|-------|----------|
/// | 0 | Fastest | 1:1 | Testing/debugging |
/// | 1-3 | Fast | 2:1-3:1 | Real-time systems |
/// | 4-6 | Medium | 3:1-5:1 | General use |
/// | 7-9 | Slow | 5:1-7:1 | Bandwidth-constrained |
///
/// ## Example Usage
///
/// ```rust,ignore
/// let compressor = MessageCompressor::new(6, 10);
///
/// // Compress messages
/// let messages = vec![message1, message2, message3];
/// let compressed = compressor.compress_messages(messages)?;
///
/// // Decompress messages
/// let decompressed = compressor.decompress_messages(&compressed)?;
/// ```
pub struct MessageCompressor {
	/// Gzip compression level (0-9, where 0 is no compression)
	CompressionLevel:u32,

	/// Minimum number of messages required for batch processing
	BatchSize:usize,
}

impl MessageCompressor {
	/// Create a new message compressor with specified parameters
	///
	/// ## Parameters
	/// - `CompressionLevel`: Gzip compression level (0-9, default 6)
	/// - `BatchSize`: Minimum messages for batch processing (default 10)
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let compressor = MessageCompressor::new(6, 10);
	/// ```
	pub fn new(CompressionLevel:u32, BatchSize:usize) -> Self {
		dev_log!(
			"encryption",
			"[MessageCompressor] Created with level: {}, batch size: {}",
			CompressionLevel,
			BatchSize
		);

		Self { CompressionLevel, BatchSize }
	}

	/// Compress messages using Gzip for efficient transfer
	///
	/// This method serializes multiple messages to JSON and compresses them
	/// using Gzip, significantly reducing the payload size.
	///
	/// ## Parameters
	/// - `Messages`: Vector of TauriIPCMessage to compress
	///
	/// ## Returns
	/// - `Ok(Vec<u8>)`: Compressed message data
	/// - `Err(String)`: Error message if compression fails
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let messages = vec![msg1, msg2, msg3];
	/// let compressed = compressor.compress_messages(messages)?;
	/// ```
	pub fn compress_messages(&self, Messages:Vec<TauriIPCMessage>) -> Result<Vec<u8>, String> {
		dev_log!("encryption", "[MessageCompressor] Compressing {} messages", Messages.len());

		// Serialize messages to JSON
		let SerializedMessages =
			serde_json::to_vec(&Messages).map_err(|e| format!("Failed to serialize messages: {}", e))?;

		let original_size = SerializedMessages.len();

		// Compress using Gzip
		let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.CompressionLevel));

		encoder
			.write_all(&SerializedMessages)
			.map_err(|e| format!("Failed to compress messages: {}", e))?;

		let compressed_data = encoder.finish().map_err(|e| format!("Failed to finish compression: {}", e))?;

		let compressed_size = compressed_data.len();

		let ratio = if original_size > 0 {
			(compressed_size as f64 / original_size as f64) * 100.0
		} else {
			100.0
		};

		dev_log!(
			"encryption",
			"[MessageCompressor] Compression complete: {} -> {} bytes ({:.1}%)",
			original_size,
			compressed_size,
			ratio
		);

		Ok(compressed_data)
	}

	/// Decompress messages from compressed data
	///
	/// This method decompresses Gzip-compressed data and deserializes it back
	/// into TauriIPCMessage objects.
	///
	/// ## Parameters
	/// - `CompressedData`: Compressed message data
	///
	/// ## Returns
	/// - `Ok(Vec<TauriIPCMessage>)`: Decompressed messages
	/// - `Err(String)`: Error message if decompression fails
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// let messages = compressor.decompress_messages(&compressed_data)?;
	/// ```
	pub fn decompress_messages(&self, CompressedData:&[u8]) -> Result<Vec<TauriIPCMessage>, String> {
		dev_log!("encryption", "[MessageCompressor] Decompressing {} bytes", CompressedData.len());

		let compressed_size = CompressedData.len();

		// Decompress using Gzip
		let mut decoder = GzDecoder::new(CompressedData);

		let mut DecompressedData = Vec::new();

		decoder
			.read_to_end(&mut DecompressedData)
			.map_err(|e| format!("Failed to decompress data: {}", e))?;

		let decompressed_size = DecompressedData.len();

		// Deserialize messages with explicit type annotation
		let messages:Vec<TauriIPCMessage> =
			serde_json::from_slice(&DecompressedData).map_err(|e| format!("Failed to deserialize messages: {}", e))?;

		dev_log!(
			"encryption",
			"[MessageCompressor] Decompression complete: {} -> {} bytes, {} messages",
			compressed_size,
			decompressed_size,
			messages.len()
		);

		Ok(messages)
	}

	/// Check if messages should be batched for compression
	///
	/// This method determines if the number of messages meets the threshold
	/// for batch compression.
	///
	/// ## Parameters
	/// - `MessagesCount`: Number of messages to check
	///
	/// ## Returns
	/// - `true`: Should batch (meets minimum threshold)
	/// - `false`: Should not batch (below threshold)
	///
	/// ## Example
	///
	/// ```rust,ignore
	/// if compressor.should_batch(messages.len()) {
	///     // Batch compress
	/// } else {
	///     // Send individually
	/// }
	/// ```
	pub fn should_batch(&self, MessagesCount:usize) -> bool {
		let should_batch = MessagesCount >= self.BatchSize;

		dev_log!(
			"encryption",
			"[MessageCompressor] Batch check: {} >= {} = {}",
			MessagesCount,
			self.BatchSize,
			should_batch
		);

		should_batch
	}

	/// Get the compression level
	pub fn compression_level(&self) -> u32 { self.CompressionLevel }

	/// Get the batch size threshold
	pub fn batch_size(&self) -> usize { self.BatchSize }

	/// Create a compressor with default settings (level 6, batch size 10)
	pub fn default() -> Self { Self::new(6, 10) }

	/// Create a fast compressor (level 3, batch size 5)
	pub fn fast() -> Self { Self::new(3, 5) }

	/// Create a maximum compression compressor (level 9, batch size 20)
	pub fn max() -> Self { Self::new(9, 20) }
}

#[cfg(test)]
mod tests {

	use super::*;

	fn create_test_message(id:u32) -> TauriIPCMessage {
		TauriIPCMessage::new(
			format!("test_channel_{}", id),
			serde_json::json!({
				"id": id,
				"data": "test data that should compress well when repeated many times across multiple messages".repeat(10)
			}),
			Some("test_sender".to_string()),
		)
	}

	#[test]
	fn test_compressor_creation() {
		let compressor = MessageCompressor::new(6, 10);

		assert_eq!(compressor.compression_level(), 6);

		assert_eq!(compressor.batch_size(), 10);
	}

	#[test]
	fn test_default_compressor() {
		let compressor = MessageCompressor::default();

		assert_eq!(compressor.compression_level(), 6);

		assert_eq!(compressor.batch_size(), 10);
	}

	#[test]
	fn test_fast_compressor() {
		let compressor = MessageCompressor::fast();

		assert_eq!(compressor.compression_level(), 3);

		assert_eq!(compressor.batch_size(), 5);
	}

	#[test]
	fn test_max_compressor() {
		let compressor = MessageCompressor::max();

		assert_eq!(compressor.compression_level(), 9);

		assert_eq!(compressor.batch_size(), 20);
	}

	#[test]
	fn test_should_batch() {
		let compressor = MessageCompressor::new(6, 10);

		assert!(!compressor.should_batch(5));

		assert!(compressor.should_batch(10));

		assert!(compressor.should_batch(15));
	}

	#[test]
	fn test_compress_and_decompress() {
		let compressor = MessageCompressor::default();

		let original_messages = vec![create_test_message(1), create_test_message(2), create_test_message(3)];

		// Compress
		let compressed = compressor.compress_messages(original_messages.clone()).unwrap();

		assert!(!compressed.is_empty());

		// Decompress
		let decompressed = compressor.decompress_messages(&compressed).unwrap();

		assert_eq!(decompressed.len(), original_messages.len());

		// Verify content
		for i in 0..original_messages.len() {
			assert_eq!(decompressed[i].channel, original_messages[i].channel);
		}
	}

	#[test]
	fn test_compression_ratio() {
		let compressor = MessageCompressor::default();

		// Create large messages that should compress well
		let messages:Vec<TauriIPCMessage> = (0..20).map(|i| create_test_message(i)).collect();

		let compressed = compressor.compress_messages(messages.clone()).unwrap();

		// Compressed size should be significantly smaller
		let original_data = serde_json::to_vec(&messages).unwrap();

		assert!(compressed.len() < original_data.len());
	}

	#[test]
	fn test_empty_messages() {
		let compressor = MessageCompressor::default();

		let messages = vec![];

		let compressed = compressor.compress_messages(messages).unwrap();

		let decompressed = compressor.decompress_messages(&compressed).unwrap();

		assert!(decompressed.is_empty());
	}

	#[test]
	fn test_single_message() {
		let compressor = MessageCompressor::default();

		let messages = vec![create_test_message(1)];

		let compressed = compressor.compress_messages(messages.clone()).unwrap();

		let decompressed = compressor.decompress_messages(&compressed).unwrap();

		assert_eq!(decompressed.len(), 1);

		assert_eq!(decompressed[0].channel, messages[0].channel);
	}
}
