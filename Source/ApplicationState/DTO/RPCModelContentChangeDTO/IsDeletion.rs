//! `RPCModelContentChangeDTO::IsDeletion`

use super::Struct;
use serde::Deserialize;
use super::RPCRangeDTO::Struct;

pub fn Fn(This:&Struct) -> bool { This.Text.is_empty() }
