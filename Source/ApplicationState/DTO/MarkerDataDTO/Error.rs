//! `MarkerDataDTO::Error`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::MarkerSeverity::MarkerSeverity;

pub fn Fn(Message:String, LineNumber:u32, Column:u32) -> Struct {
		Self {
			Severity:MarkerSeverity::Error as u32,

			Message,

			StartLineNumber:LineNumber,

			StartColumn:Column,

			EndLineNumber:LineNumber,

			EndColumn:Column,
			..Default::default()
		}
	}
