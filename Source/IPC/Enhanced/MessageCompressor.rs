//! # Message Compressor and Batching
//!
//! Advanced message compression and batching for IPC performance optimization.
//! Supports Brotli compression for large payloads and intelligent message
//! batching.

use std::{collections::VecDeque, time::Duration};

use brotli::{BrotliEncoderParams, CompressorReader, CompressorWriter};
use flate2::{
	Compression,
	write::{GzEncoder, ZlibEncoder},
};
use log::{debug, error, info, trace};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

/// Message compression levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionLevel {
	Fast = 1,
	Balanced = 6,
	High = 11,
}

/// Compression algorithm selection
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
	Brotli,
	Gzip,
	Zlib,
}

/// Message batch configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
	pub max_batch_size:usize,
	pub max_batch_delay_ms:u64,
	pub compression_threshold_bytes:usize,
	pub compression_level:CompressionLevel,
	pub algorithm:CompressionAlgorithm,
}

impl Default for BatchConfig {
	fn default() -> Self {
		Self {
			max_batch_size:100,
			max_batch_delay_ms:100,
			compression_threshold_bytes:1024, // 1KB
			compression_level:CompressionLevel::Balanced,
			algorithm:CompressionAlgorithm::Brotli,
		}
	}
}

/// Message compressor with batching capabilities
pub struct MessageCompressor {
	config:BatchConfig,
	current_batch:VecDeque<Vec<u8>>,
	batch_start_time:Option<Instant>,
	batch_size_bytes:usize,
}

impl MessageCompressor {
	/// Create a new message compressor with configuration
	pub fn new(config:BatchConfig) -> Self {
		Self { config, current_batch:VecDeque::new(), batch_start_time:None, batch_size_bytes:0 }
	}

	/// Add a message to the current batch
	pub fn add_message(&mut self, message_data:&[u8]) -> bool {
		let message_size = message_data.len();
		let should_compress = message_size >= self.config.compression_threshold_bytes;

		// Check if we should flush based on size
		if self.batch_size_bytes + message_size > self.config.max_batch_size * 1024 {
			return false; // Batch is full
		}

		// Add message to batch
		self.current_batch.push_back(message_data.to_vec());
		self.batch_size_bytes += message_size;

		// Initialize batch timer if this is the first message
		if self.batch_start_time.is_none() {
			self.batch_start_time = Some(Instant::now());
		}

		true
	}

	/// Check if batch should be flushed
	pub fn should_flush(&self) -> bool {
		if self.current_batch.is_empty() {
			return false;
		}

		// Check batch size limit
		if self.current_batch.len() >= self.config.max_batch_size {
			return true;
		}

		// Check time limit
		if let Some(start_time) = self.batch_start_time {
			let elapsed = start_time.elapsed();
			if elapsed.as_millis() >= self.config.max_batch_delay_ms as u128 {
				return true;
			}
		}

		false
	}

	/// Compress and flush the current batch
	pub fn flush_batch(&mut self) -> Result<CompressedBatch, String> {
		if self.current_batch.is_empty() {
			return Err("No messages in batch to flush".to_string());
		}

		let batch_messages:Vec<Vec<u8>> = self.current_batch.drain(..).collect();
		let total_size = self.batch_size_bytes;

		// Reset batch state
		self.batch_start_time = None;
		self.batch_size_bytes = 0;

		// Serialize batch
		let serialized_batch =
			bincode::serialize(&batch_messages).map_err(|e| format!("Failed to serialize batch: {}", e))?;

		// Compress if needed
		let (compressed_data, compression_info) = if total_size >= self.config.compression_threshold_bytes {
			self.compress_data(&serialized_batch).map(|(data, info)| (Some(data), info))
		} else {
			(None, CompressionInfo::none())
		}?;

		Ok(CompressedBatch {
			messages_count:batch_messages.len(),
			original_size:total_size,
			compressed_size:compressed_data.as_ref().map(|d| d.len()).unwrap_or(total_size),
			compressed_data,
			compression_info,
			timestamp:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		})
	}

	/// Compress data using configured algorithm
	fn compress_data(&self, data:&[u8]) -> Result<(Vec<u8>, CompressionInfo), String> {
		match self.config.algorithm {
			CompressionAlgorithm::Brotli => self.compress_brotli(data),
			CompressionAlgorithm::Gzip => self.compress_gzip(data),
			CompressionAlgorithm::Zlib => self.compress_zlib(data),
		}
	}

	/// Compress using Brotli algorithm
	fn compress_brotli(&self, data:&[u8]) -> Result<(Vec<u8>, CompressionInfo), String> {
		let mut params = BrotliEncoderParams::default();
		params.quality = self.config.compression_level as i32;

		let mut compressed = Vec::new();
		let mut writer = CompressorWriter::with_params(&mut compressed, data.len(), &params);

		std::io::Write::write_all(&mut writer, data).map_err(|e| format!("Brotli compression failed: {}", e))?;

		writer.flush().map_err(|e| format!("Brotli flush failed: {}", e))?;

		let ratio = data.len() as f64 / compressed.len() as f64;

		Ok((
			compressed,
			CompressionInfo {
				algorithm:"brotli".to_string(),
				level:self.config.compression_level as u32,
				ratio,
			},
		))
	}

	/// Compress using Gzip algorithm
	fn compress_gzip(&self, data:&[u8]) -> Result<(Vec<u8>, CompressionInfo), String> {
		let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.config.compression_level as u32));
		encoder.write_all(data).map_err(|e| format!("Gzip compression failed: {}", e))?;

		let compressed = encoder.finish().map_err(|e| format!("Gzip finish failed: {}", e))?;

		let ratio = data.len() as f64 / compressed.len() as f64;

		Ok((
			compressed,
			CompressionInfo { algorithm:"gzip".to_string(), level:self.config.compression_level as u32, ratio },
		))
	}

	/// Compress using Zlib algorithm
	fn compress_zlib(&self, data:&[u8]) -> Result<(Vec<u8>, CompressionInfo), String> {
		let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(self.config.compression_level as u32));
		encoder.write_all(data).map_err(|e| format!("Zlib compression failed: {}", e))?;

		let compressed = encoder.finish().map_err(|e| format!("Zlib finish failed: {}", e))?;

		let ratio = data.len() as f64 / compressed.len() as f64;

		Ok((
			compressed,
			CompressionInfo { algorithm:"zlib".to_string(), level:self.config.compression_level as u32, ratio },
		))
	}

	/// Decompress a batch
	pub fn decompress_batch(&self, batch:&CompressedBatch) -> Result<Vec<Vec<u8>>, String> {
		let data = if let Some(ref compressed_data) = batch.compressed_data {
			self.decompress_data(compressed_data, &batch.compression_info.algorithm)
		} else {
			Ok(bincode::serialize(&batch).map_err(|e| format!("Serialization failed: {}", e))?)
		}?;

		bincode::deserialize(&data).map_err(|e| format!("Failed to deserialize batch: {}", e))
	}

	/// Decompress data using specified algorithm
	fn decompress_data(&self, data:&[u8], algorithm:&str) -> Result<Vec<u8>, String> {
		match algorithm {
			"brotli" => self.decompress_brotli(data),
			"gzip" => self.decompress_gzip(data),
			"zlib" => self.decompress_zlib(data),
			_ => Err(format!("Unsupported compression algorithm: {}", algorithm)),
		}
	}

	/// Decompress Brotli data
	fn decompress_brotli(&self, data:&[u8]) -> Result<Vec<u8>, String> {
		let mut decompressed = Vec::new();
		let mut reader = CompressorReader::new(data, data.len());

		std::io::Read::read_to_end(&mut reader, &mut decompressed)
			.map_err(|e| format!("Brotli decompression failed: {}", e))?;

		Ok(decompressed)
	}

	/// Decompress Gzip data
	fn decompress_gzip(&self, data:&[u8]) -> Result<Vec<u8>, String> {
		use flate2::read::GzDecoder;

		let mut decoder = GzDecoder::new(data);
		let mut decompressed = Vec::new();
		decoder
			.read_to_end(&mut decompressed)
			.map_err(|e| format!("Gzip decompression failed: {}", e))?;

		Ok(decompressed)
	}

	/// Decompress Zlib data
	fn decompress_zlib(&self, data:&[u8]) -> Result<Vec<u8>, String> {
		use flate2::read::ZlibDecoder;

		let mut decoder = ZlibDecoder::new(data);
		let mut decompressed = Vec::new();
		decoder
			.read_to_end(&mut decompressed)
			.map_err(|e| format!("Zlib decompression failed: {}", e))?;

		Ok(decompressed)
	}

	/// Get current batch statistics
	pub fn get_batch_stats(&self) -> BatchStats {
		BatchStats {
			messages_count:self.current_batch.len(),
			total_size_bytes:self.batch_size_bytes,
			batch_age_ms:self.batch_start_time.map(|t| t.elapsed().as_millis() as u64).unwrap_or(0),
		}
	}

	/// Clear current batch without flushing
	pub fn clear_batch(&mut self) {
		self.current_batch.clear();
		self.batch_start_time = None;
		self.batch_size_bytes = 0;
	}
}

/// Compressed batch structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedBatch {
	pub messages_count:usize,
	pub original_size:usize,
	pub compressed_size:usize,
	pub compressed_data:Option<Vec<u8>>,
	pub compression_info:CompressionInfo,
	pub timestamp:u64,
}

/// Compression information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionInfo {
	pub algorithm:String,
	pub level:u32,
	pub ratio:f64,
}

impl CompressionInfo {
	fn none() -> Self { Self { algorithm:"none".to_string(), level:0, ratio:1.0 } }
}

/// Batch statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchStats {
	pub messages_count:usize,
	pub total_size_bytes:usize,
	pub batch_age_ms:u64,
}

/// Utility functions for message compression
impl MessageCompressor {
	/// Compress a single message
	pub fn compress_single_message(
		message_data:&[u8],
		algorithm:CompressionAlgorithm,
		level:CompressionLevel,
	) -> Result<(Vec<u8>, CompressionInfo), String> {
		let config = BatchConfig { algorithm, compression_level:level, ..Default::default() };

		let compressor = MessageCompressor::new(config);
		compressor.compress_data(message_data)
	}

	/// Calculate compression ratio
	pub fn calculate_compression_ratio(original_size:usize, compressed_size:usize) -> f64 {
		if compressed_size == 0 {
			return 0.0;
		}
		original_size as f64 / compressed_size as f64
	}

	/// Estimate compression savings
	pub fn estimate_savings(original_size:usize, expected_ratio:f64) -> usize {
		(original_size as f64 * (1.0 - 1.0 / expected_ratio)) as usize
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_message_compression() {
		let test_data = b"This is a test message for compression evaluation".to_vec();

		// Test Brotli compression
		let (compressed, info) = MessageCompressor::compress_single_message(
			&test_data,
			CompressionAlgorithm::Brotli,
			CompressionLevel::Balanced,
		)
		.unwrap();

		assert!(compressed.len() < test_data.len());
		assert!(info.ratio > 1.0);
		assert_eq!(info.algorithm, "brotli");

		// Test Gzip compression
		let (compressed_gzip, info_gzip) = MessageCompressor::compress_single_message(
			&test_data,
			CompressionAlgorithm::Gzip,
			CompressionLevel::Balanced,
		)
		.unwrap();

		assert!(compressed_gzip.len() < test_data.len());
		assert!(info_gzip.ratio > 1.0);
		assert_eq!(info_gzip.algorithm, "gzip");
	}

	#[test]
	fn test_batch_compression() {
		let config = BatchConfig::default();
		let mut compressor = MessageCompressor::new(config);

		let messages:Vec<Vec<u8>> = (0..5).map(|i| format!("Message {}", i).into_bytes()).collect();

		for message in &messages {
			compressor.add_message(message);
		}

		assert!(compressor.should_flush());

		let batch = compressor.flush_batch().unwrap();
		assert_eq!(batch.messages_count, 5);
		assert!(batch.compressed_size <= batch.original_size);
	}

	#[test]
	fn test_compression_ratio_calculation() {
		let ratio = MessageCompressor::calculate_compression_ratio(1000, 500);
		assert_eq!(ratio, 2.0);

		let savings = MessageCompressor::estimate_savings(1000, 2.0);
		assert_eq!(savings, 500);
	}
}
