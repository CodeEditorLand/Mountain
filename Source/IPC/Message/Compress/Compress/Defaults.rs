//! `Compress::Defaults`

use super::Struct;
use std::io::{Read, Write};
use flate2::{Compression, read::GzDecoder, write::GzEncoder};
use serde::Serialize;
use super::super::Define::DefineMessage::TauriIPCMessage;
use crate::dev_log;

pub fn Fn() -> Struct { Struct::new(6, 10, 1024) }
