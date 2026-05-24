//! `Compress::ShouldBatch`

use super::Struct;
use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Serialize;
use super::super::Define::DefineMessage::TauriIPCMessage;
use crate::dev_log;

pub fn Fn(This:&Struct, MessagesCount:usize) -> bool { MessagesCount >= This.BatchSize }
