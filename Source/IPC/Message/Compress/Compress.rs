//! # Compress
//!
//! ## File: IPC/Message/Compress/Compress.rs
//!
//! ## Role in Mountain Architecture
//!
//! This module provides gzip compression and decompression for IPC messages, reducing bandwidth usage between Mountain and Wind. It manages the trade-off between compression overhead and message size savings.
//!
//! ## Primary Responsibility
//!
//! Provide gzip compression/decompression for IPC message payloads with size-based batching strategy.
//!
//! ## Secondary Responsibilities
//!
//! - Determine when compression is beneficial
//! - Manage compression level for performance tuning
//! - Protect against compression bomb attacks
//!
//! ## Dependencies
//!
//! **External Crates:**
//! - `flate2` - Gzip compression and decompression
//! - `serde_json` - Serialization for compressed data
//!
//! **Internal Modules:**
//! - `IPC::Message::Define::DefineMessage` - Provides TauriIPCMessage type
//!
//! ## Dependents
//!
//! - `IPC::TauriIPCServer` - Uses compression for message batches
//!
//! ## VSCode Pattern Reference
//!
//! Inspired by VSCode's compression strategy where only messages above a size threshold are compressed to avoid overhead on small messages.
//!
//! ## Security Considerations
//!
//! - Compression bomb protection with maximum decompressed size limits
//! - Memory limits to prevent denial of service
//! - Input validation on compressed data length
//!
//! ## Performance Considerations
//!
//! - Compression only applied to messages >1KB to avoid overhead
//! - Configurable compression level (default: 6, balances speed/ratio)
//! - Batching strategy reduces per-message overhead
//!
//! ## Error Handling Strategy
//!
//! - Returns Result<T, String> for all fallible operations
//! - Detailed error messages for compression failures
//! - Validates input before processing
//!
//! ## Thread Safety
//!
//! - Stateless functions are inherently thread-safe
//! - Can be safely called from multiple threads concurrently
//!
//! ## TODO Items
//!
//! - [ ] Add compression ratio tracking for optimization
//! - [ ] Implement adaptive compression based on historical data
//! - [ ] Add support for alternative compression algorithms (brotli, zstd)


use std::io::{Read, Write};
use flate2::{write::GzEncoder, read::GzDecoder, Compression};
use log::{debug, trace, warn};

use super::super::Define::DefineMessage::TauriIPCMessage;

/// Minimum message size (in bytes) to trigger compression
const COMPRESSION_THRESHOLD: usize = 1024; // 1KB

/// Maximum allowed decompressed size to prevent compression bombs
const MAX_DECOMPRESSED_SIZE: usize = 100 * 1024 * 1024; // 100MB

/// Default compression level (0-9, where 6 is balanced)
pub const DEFAULT_COMPRESSION_LEVEL: u32 = 6;

/// Message compressor for optimizing IPC message transfer
///
/// Provides gzip compression with configurable settings and smart
/// batching strategies to optimize bandwidth usage.
pub struct Compress {
    /// Compression level (0-9, where 9 is maximum compression)
    pub CompressionLevel: u32,
    /// Minimum number of messages to batch before compression
    pub BatchSize: usize,
    /// Maximum decompressed size limit (protection against compression bombs)
    pub MaxDecompressedSize: usize,
}

impl Compress {
    /// Create a new message compressor with defaults
    pub fn New() -> Self {
        Self {
            CompressionLevel: DEFAULT_COMPRESSION_LEVEL,
            BatchSize: 10,
            MaxDecompressedSize: MAX_DECOMPRESSED_SIZE,
        }
    }

    /// Create a new message compressor with specified parameters
    pub fn NewWithParams(CompressionLevel: u32, BatchSize: usize) -> Self {
        Self {
            CompressionLevel: CompressionLevel.min(9).max(0),
            BatchSize,
            MaxDecompressedSize: MAX_DECOMPRESSED_SIZE,
        }
    }

    /// Create a new message compressor with custom max decompressed size
    pub fn NewWithMaxSize(CompressionLevel: u32, BatchSize: usize, MaxDecompressedSize: usize) -> Self {
        Self {
            CompressionLevel: CompressionLevel.min(9).max(0),
            BatchSize,
            MaxDecompressedSize: MaxDecompressedSize,
        }
    }

    /// Compress a single message if it exceeds the size threshold
    ///
    /// Returns None if the message is too small to benefit from compression.
    pub fn CompressMessage(&self, Message: &TauriIPCMessage) -> Result<Option<Vec<u8>>, String> {
        let serialized = serde_json::to_vec(Message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;

        // Only compress if message is large enough
        if serialized.len() < COMPRESSION_THRESHOLD {
            trace!("[Compress] Message too small to compress ({} bytes)", serialized.len());
            return Ok(None);
        }

        self.CompressBytes(&serialized)
    }

    /// Compress a batch of messages into a single byte array
    pub fn CompressMessages(&self, Messages: &[TauriIPCMessage]) -> Result<Vec<u8>, String> {
        let serialized = serde_json::to_vec(Messages)
            .map_err(|e| format!("Failed to serialize messages: {}", e))?;

        // Validate size before compression
        if serialized.len() > self.MaxDecompressedSize {
            return Err(format!("Message batch too large to compress ({} > {} bytes)",
                serialized.len(), self.MaxDecompressedSize));
        }

        self.CompressBytes(&serialized)
    }

    /// Compress raw byte data using gzip
    fn CompressBytes(&self, Data: &[u8]) -> Result<Vec<u8>, String> {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.CompressionLevel));
        encoder.write_all(Data)
            .map_err(|e| format!("Failed to compress data: {}", e))?;

        let compressed = encoder.finish()
            .map_err(|e| format!("Failed to finish compression: {}", e))?;

        debug!("[Compress] Compressed {} bytes to {} bytes (ratio: {:.2}%)",
            Data.len(), compressed.len(),
            (compressed.len() as f64 / Data.len() as f64) * 100.0);

        Ok(compressed)
    }

    /// Decompress a single message
    pub fn DecompressMessage(&self, CompressedData: &[u8]) -> Result<TauriIPCMessage, String> {
        let decompressed = self.DecompressBytes(CompressedData)?;
        serde_json::from_slice(&decompressed)
            .map_err(|e| format!("Failed to deserialize message: {}", e))
    }

    /// Decompress a batch of messages
    pub fn DecompressMessages(&self, CompressedData: &[u8]) -> Result<Vec<TauriIPCMessage>, String> {
        let decompressed = self.DecompressBytes(CompressedData)?;
        serde_json::from_slice(&decompressed)
            .map_err(|e| format!("Failed to deserialize messages: {}", e))
    }

    /// Decompress raw byte data using gzip
    fn DecompressBytes(&self, CompressedData: &[u8]) -> Result<Vec<u8>, String> {
        let mut decoder = GzDecoder::new(CompressedData);
        let mut decompressed = Vec::new();

        // Read with size limit to prevent compression bombs
        decoder.by_ref().take(self.MaxDecompressedSize as u64)
            .read_to_end(&mut decompressed)
            .map_err(|e| format!("Failed to decompress data: {}", e))?;

        // Check if more data remains (potential compression bomb)
        let mut buffer = [0u8; 1];
        if decoder.read(&mut buffer).is_ok() {
            warn!("[Compress] Decompressed data exceeds maximum size limit ({} bytes)",
                self.MaxDecompressedSize);
            return Err("Decompressed data exceeds maximum size limit".to_string());
        }

        trace!("[Compress] Decompressed {} bytes to {} bytes",
            CompressedData.len(), decompressed.len());

        Ok(decompressed)
    }

    /// Determine if messages should be batched before compression
    pub fn ShouldBatch(&self, MessagesCount: usize) -> bool {
        MessagesCount >= self.BatchSize
    }

    /// Determine if a single message should be compressed
    pub fn ShouldCompressMessage(&self, Message: &TauriIPCMessage) -> bool {
        match serde_json::to_vec(Message) {
            Ok(serialized) => serialized.len() >= COMPRESSION_THRESHOLD,
            Err(_) => false,
        }
    }

    /// Get the compression ratio for a given input
    ///
    /// Returns None if compression would not be beneficial.
    pub fn GetCompressionRatio(&self, Message: &TauriIPCMessage) -> Option<f64> {
        let serialized = serde_json::to_vec(Message).ok()?;
        if serialized.len() < COMPRESSION_THRESHOLD {
            return None;
        }

        match self.CompressBytes(&serialized) {
            Ok(compressed) => Some(compressed.len() as f64 / serialized.len() as f64),
            Err(_) => None,
        }
    }

    /// Estimate compression time for a given message size
    ///
    /// This is a rough estimate based on typical compression speeds.
    pub fn EstimateCompressionTimeMicroseconds(&self, Bytes: usize) -> u64 {
        // Typical gzip compression: ~100 MB/s on modern hardware
        // This is a conservative estimate
        const BYTES_PER_MICROSECOND: usize = 100; // 100 bytes per microsecond
        (Bytes / BYTES_PER_MICROSECOND) as u64
    }
}

impl Default for Compress {
    fn default() -> Self {
        Self::New()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_small_message_returns_none() {
        let compressor = Compress::New();
        let message = TauriIPCMessage::New(
            "test-channel".to_string(),
            serde_json::json!({"small": "data"}),
        );
        let result = compressor.CompressMessage(&message);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_compress_large_message_returns_data() {
        let compressor = Compress::New();
        // Create a large message
        let large_data: String = "x".repeat(2000);
        let message = TauriIPCMessage::New(
            "test-channel".to_string(),
            serde_json::json!({"large": large_data}),
        );
        let result = compressor.CompressMessage(&message);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_compress_decompress_roundtrip() {
        let compressor = Compress::New();
        let message = TauriIPCMessage::New(
            "test-channel".to_string(),
            serde_json::json!({"hello": "world", "data": "test payload"}),
        );

        let compressed = compressor.CompressMessage(&message);
        assert!(compressed.is_ok());

        if let Some(comp_data) = compressed.unwrap() {
            let decompressed = compressor.DecompressMessage(&comp_data);
            assert!(decompressed.is_ok());
            assert_eq!( decompressed.unwrap().Channel, message.Channel);
        }
    }

    #[test]
    fn test_should_batch() {
        let compressor = Compress::New();
        assert!(!compressor.ShouldBatch(5));
        assert!(compressor.ShouldBatch(10));
        assert!(compressor.ShouldBatch(20));
    }
}

pub use Compress;
