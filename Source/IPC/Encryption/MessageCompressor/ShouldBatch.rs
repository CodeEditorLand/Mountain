//! `MessageCompressor::ShouldBatch`

use super::Struct;
use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct, MessagesCount:usize) -> bool {
		let should_batch = MessagesCount >= This.BatchSize;

		dev_log!(
			"encryption",
			"[MessageCompressor] Batch check: {} >= {} = {}",
			MessagesCount,
			This.BatchSize,
			should_batch
		);

		should_batch
	}
