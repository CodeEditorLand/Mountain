//! `RPCRangeDTO::IsEmpty`

use super::Struct;
use serde::Deserialize;

pub fn Fn(This:&Struct) -> bool { This.StartLineNumber == This.EndLineNumber && This.StartColumn == This.EndColumn }
