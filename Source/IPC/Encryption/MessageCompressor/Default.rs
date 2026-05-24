//! `MessageCompressor::Default`

use super::Struct;
use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use super::super::Message::Types::TauriIPCMessage;
use crate::dev_log;

pub fn Fn() -> Struct { Struct::new(6, 10) }
