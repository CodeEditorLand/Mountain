//! `RPCRangeDTO::LineRange`

use super::Struct;
use serde::Deserialize;

pub fn Fn(LineNumber:usize, StartColumn:usize, EndColumn:usize) -> Result<Self, String> {
		Struct::New(LineNumber, StartColumn, LineNumber, EndColumn)
	}
