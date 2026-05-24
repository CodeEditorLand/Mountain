//! `RPCModelContentChangeDTO::IsReplacement`

use super::Struct;
use serde::Deserialize;
use super::RPCRangeDTO::Struct;

pub fn Fn(This:&Struct) -> bool { !This.Range.IsEmpty() && !This.Text.is_empty() }
