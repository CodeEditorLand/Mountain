//! `OutputChannelStateDTO::GetFormattedBufferSize`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> String { FormatBytes(This.Buffer.len()) }
