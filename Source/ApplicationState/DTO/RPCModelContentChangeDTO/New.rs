//! `RPCModelContentChangeDTO::New`

use super::Struct;
use serde::Deserialize;
use super::RPCRangeDTO::Struct;

pub fn Fn(Range:RPCRangeDTO, Text:String) -> Result<Self, String> {
		// Text is allowed to be empty (for deletion operations)
		Ok(Self { Range, Text })
	}
