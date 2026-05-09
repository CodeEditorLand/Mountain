#![allow(non_snake_case)]

//! `Compressor::Struct` - message batching + compression
//! engine. Buffers messages until size or time triggers a
//! flush, then emits a `CompressedBatch::Struct` using the
//! configured algorithm. Struct + 14-method impl + utility
//! functions stay in one file - tightly coupled cluster.

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

use crate::IPC::Enhanced::MessageCompressor::{
	BatchConfig::Struct as BatchConfig,
	BatchStats::Struct as BatchStats,
	CompressedBatch::Struct as CompressedBatch,
	CompressionAlgorithm::Enum as CompressionAlgorithm,
	CompressionInfo::Struct as CompressionInfo,
	CompressionLevel::Enum as CompressionLevel,
};

pub struct Struct {
	pub(super) Config:BatchConfig,

	pub(super) CurrentBatch:VecDeque<Vec<u8>>,

	pub(super) BatchStartTime:Option<Instant>,

	pub(super) BatchSizeBytes:usize,
}

impl Struct {
	pub fn new(config:BatchConfig) -> Self {
		Self {
			Config:config,

			CurrentBatch:VecDeque::new(),

			BatchStartTime:None,

			BatchSizeBytes:0,
		}
	}

	pub fn add_message(&mut self, MessageData:&[u8]) -> bool {
		let MessageSize = MessageData.len();

		let _should_compress = MessageSize >= self.Config.CompressionThresholdBytes;

		if self.BatchSizeBytes + MessageSize > self.Config.MaxBatchSize * 1024 {
			return false;
		}

		self.CurrentBatch.push_back(MessageData.to_vec());

		self.BatchSizeBytes += MessageSize;

		if self.BatchStartTime.is_none() {
			self.BatchStartTime = Some(Instant::now());
		}

		true
	}

	pub fn should_flush(&self) -> bool {
		if self.CurrentBatch.is_empty() {
			return false;
		}

		if self.CurrentBatch.len() >= self.Config.MaxBatchSize {
			return true;
		}

		if let Some(start_time) = self.BatchStartTime {
			let elapsed = start_time.elapsed();

			if elapsed.as_millis() >= self.Config.MaxBatchDelayMs as u128 {
				return true;
			}
		}

		false
	}

	pub fn flush_batch(&mut self) -> Result<CompressedBatch, String> {
		if self.CurrentBatch.is_empty() {
			return Err("No messages in batch to flush".to_string());
		}

		let BatchMessages:Vec<Vec<u8>> = self.CurrentBatch.drain(..).collect();

		let total_size = self.BatchSizeBytes;

		self.BatchStartTime = None;

		self.BatchSizeBytes = 0;

		let config = bincode::config::standard();

		let serialized_batch =
			encode_to_vec(&BatchMessages, config).map_err(|e| format!("Failed to serialize batch: {}", e))?;

		let (CompressedData, compression_info) = if total_size >= self.Config.CompressionThresholdBytes {
			self.compress_data(&serialized_batch).map(|(data, info)| (Some(data), info))
		} else {
			Ok((None, CompressionInfo::none()))
		}?;

		Ok(CompressedBatch {
			messages_count:BatchMessages.len(),
			original_size:total_size,
			compressed_size:CompressedData.as_ref().map(|d| d.len()).unwrap_or(total_size),
			compressed_data:CompressedData,
			compression_info,
			timestamp:std::time::SystemTime::now()
				.duration_since(std::time::UNIX_EPOCH)
				.unwrap_or_default()
				.as_millis() as u64,
		})
	}

	fn compress_data(&self, data:&[u8]) -> Result<(Vec<u8>, CompressionInfo), String> {
		match self.Config.Algorithm {
			CompressionAlgorithm::Brotli => self.compress_brotli(data),

			CompressionAlgorithm::Gzip => self.compress_gzip(data),

			CompressionAlgorithm::Zlib => self.compress_zlib(data),
		}
	}

	fn compress_brotli(&self, data:&[u8]) -> Result<(Vec<u8>, CompressionInfo), String> {
		let mut params = BrotliEncoderParams::default();

		params.quality = self.Config.CompressionLevel as i32;

		let mut compressed = Vec::new();

		{
			let mut writer = CompressorWriter::with_params(&mut compressed, data.len().try_into().unwrap(), &params);

			std::io::Write::write_all(&mut writer, data).map_err(|e| format!("Brotli compression failed: {}", e))?;

			writer.flush().map_err(|e| format!("Brotli flush failed: {}", e))?;
		}

		let ratio = data.len() as f64 / compressed.len() as f64;

		Ok((
			compressed,
			CompressionInfo { algorithm:"brotli".to_string(), level:self.Config.CompressionLevel as u32, ratio },
		))
	}

	fn compress_gzip(&self, data:&[u8]) -> Result<(Vec<u8>, CompressionInfo), String> {
		let mut encoder = GzEncoder::new(Vec::new(), Compression::new(self.Config.CompressionLevel as u32));

		encoder.write_all(data).map_err(|e| format!("Gzip compression failed: {}", e))?;

		let compressed = encoder.finish().map_err(|e| format!("Gzip finish failed: {}", e))?;

		let ratio = data.len() as f64 / compressed.len() as f64;

		Ok((
			compressed,
			CompressionInfo { algorithm:"gzip".to_string(), level:self.Config.CompressionLevel as u32, ratio },
		))
	}

	fn compress_zlib(&self, data:&[u8]) -> Result<(Vec<u8>, CompressionInfo), String> {
		let mut encoder = ZlibEncoder::new(Vec::new(), Compression::new(self.Config.CompressionLevel as u32));

		encoder.write_all(data).map_err(|e| format!("Zlib compression failed: {}", e))?;

		let compressed = encoder.finish().map_err(|e| format!("Zlib finish failed: {}", e))?;

		let ratio = data.len() as f64 / compressed.len() as f64;

		Ok((
			compressed,
			CompressionInfo { algorithm:"zlib".to_string(), level:self.Config.CompressionLevel as u32, ratio },
		))
	}

	pub fn decompress_batch(&self, batch:&CompressedBatch) -> Result<Vec<Vec<u8>>, String> {
		let data = if let Some(ref compressed_data) = batch.compressed_data {
			self.decompress_data(compressed_data, &batch.compression_info.algorithm)?
		} else {
			encode_to_vec(&batch, bincode::config::standard()).map_err(|e| format!("Serialization failed: {}", e))?
		};

		let (decoded, _) = decode_from_slice::<Vec<Vec<u8>>, _>(&data, bincode::config::standard())
			.map_err(|e| format!("Failed to deserialize batch: {}", e))?;

		Ok(decoded)
	}

	fn decompress_data(&self, data:&[u8], algorithm:&str) -> Result<Vec<u8>, String> {
		match algorithm {
			"brotli" => self.decompress_brotli(data),

			"gzip" => self.decompress_gzip(data),

			"zlib" => self.decompress_zlib(data),

			_ => Err(format!("Unsupported compression algorithm: {}", algorithm)),
		}
	}

	fn decompress_brotli(&self, data:&[u8]) -> Result<Vec<u8>, String> {
		let mut decompressed = Vec::new();

		let mut reader = CompressorReader::new(data, 0, data.len().try_into().unwrap(), data.len().try_into().unwrap());

		std::io::Read::read_to_end(&mut reader, &mut decompressed)
			.map_err(|e| format!("Brotli decompression failed: {}", e))?;

		Ok(decompressed)
	}

	fn decompress_gzip(&self, data:&[u8]) -> Result<Vec<u8>, String> {
		use flate2::read::GzDecoder;

		let mut decoder = GzDecoder::new(data);

		let mut decompressed = Vec::new();

		decoder
			.read_to_end(&mut decompressed)
			.map_err(|e| format!("Gzip decompression failed: {}", e))?;

		Ok(decompressed)
	}

	fn decompress_zlib(&self, data:&[u8]) -> Result<Vec<u8>, String> {
		use flate2::read::ZlibDecoder;

		let mut decoder = ZlibDecoder::new(data);

		let mut decompressed = Vec::new();

		decoder
			.read_to_end(&mut decompressed)
			.map_err(|e| format!("Zlib decompression failed: {}", e))?;

		Ok(decompressed)
	}

	pub fn get_batch_stats(&self) -> BatchStats {
		BatchStats {
			messages_count:self.CurrentBatch.len(),

			total_size_bytes:self.BatchSizeBytes,

			batch_age_ms:self.BatchStartTime.map(|t| t.elapsed().as_millis() as u64).unwrap_or(0),
		}
	}

	pub fn clear_batch(&mut self) {
		self.CurrentBatch.clear();

		self.BatchStartTime = None;

		self.BatchSizeBytes = 0;
	}

	pub fn compress_single_message(
		message_data:&[u8],

		algorithm:CompressionAlgorithm,

		level:CompressionLevel,
	) -> Result<(Vec<u8>, CompressionInfo), String> {
		let config = BatchConfig { Algorithm:algorithm, CompressionLevel:level, ..Default::default() };

		let compressor = Self::new(config);

		compressor.compress_data(message_data)
	}

	pub fn calculate_compression_ratio(original_size:usize, compressed_size:usize) -> f64 {
		if compressed_size == 0 {
			return 0.0;
		}

		original_size as f64 / compressed_size as f64
	}

	pub fn estimate_savings(original_size:usize, expected_ratio:f64) -> usize {
		(original_size as f64 * (1.0 - 1.0 / expected_ratio)) as usize
	}
}
