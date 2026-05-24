//! `RPCRangeDTO::New`

use super::Struct;
use serde::Deserialize;

pub fn Fn(StartLineNumber:usize, StartColumn:usize, EndLineNumber:usize, EndColumn:usize) -> Result<Self, String> {
		// Validate line numbers
		if StartLineNumber > MAX_LINE_NUMBER || EndLineNumber > MAX_LINE_NUMBER {
			return Err(format!("Line numbers exceed maximum of {}", MAX_LINE_NUMBER));
		}

		// Validate column numbers
		if StartColumn > MAX_COLUMN_NUMBER || EndColumn > MAX_COLUMN_NUMBER {
			return Err(format!("Column numbers exceed maximum of {}", MAX_COLUMN_NUMBER));
		}

		// Validate position consistency
		if StartLineNumber > EndLineNumber {
			return Err("Start line number cannot be greater than end line number".to_string());
		}

		// Validate column consistency within same line
		if StartLineNumber == EndLineNumber && StartColumn > EndColumn {
			return Err("Start column cannot be greater than end column on the same line".to_string());
		}

		Ok(Self { StartLineNumber, StartColumn, EndLineNumber, EndColumn })
	}
