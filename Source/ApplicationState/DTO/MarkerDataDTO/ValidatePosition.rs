//! `MarkerDataDTO::ValidatePosition`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::MarkerSeverity::MarkerSeverity;

pub fn Fn(This:&Struct) -> Result<(), String> {
		if This.StartLineNumber > This.EndLineNumber {
			return Err("Start line number cannot be greater than end line number".to_string());
		}

		if This.StartLineNumber == This.EndLineNumber && This.StartColumn > This.EndColumn {
			return Err("Start column cannot be greater than end column on the same line".to_string());
		}

		Ok(())
	}
