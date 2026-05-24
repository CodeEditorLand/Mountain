//! `RPCRangeDTO::Position`

use super::Struct;
use serde::Deserialize;

pub fn Fn(LineNumber:usize, Column:usize) -> Result<Self, String> {
		Struct::New(LineNumber, Column, LineNumber, Column)
	}
