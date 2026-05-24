//! `MessageCompressor::New`

use super::Struct;
use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(CompressionLevel:u32, BatchSize:usize) -> Struct {
		dev_log!(
			"encryption",
			"[MessageCompressor] Created with level: {}, batch size: {}",
			CompressionLevel,
			BatchSize
		);

		Self { CompressionLevel, BatchSize }
	}
