//! `OutputChannelStateDTO::GetBufferSize`

use super::Struct;
use serde::{Deserialize, Serialize};

pub fn Fn(This:&Struct) -> usize { This.Buffer.len() }
