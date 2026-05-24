//! `MarkerDataDTO::GetSeverity`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::MarkerSeverity::MarkerSeverity;

pub fn Fn(This:&Struct) -> Option<MarkerSeverity> {
		match This.Severity {
			8 => Some(MarkerSeverity::Error),

			4 => Some(MarkerSeverity::Warning),

			2 => Some(MarkerSeverity::Information),

			1 => Some(MarkerSeverity::Hint),

			_ => None,
		}
	}
