//! `MarkerDataDTO::New`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::MarkerSeverity::MarkerSeverity;

pub fn Fn(
		Severity:u32,

		Message:String,

		StartLineNumber:u32,

		StartColumn:u32,

		EndLineNumber:u32,

		EndColumn:u32,
	) -> Result<Self, String> {
		// Validate severity range
		if Severity > 8 || Severity == 0 {
			return Err("Invalid severity value: must be 1, 2, 4, or 8".to_string());
		}

		// Validate message length
		if Message.len() > MAX_MARKER_MESSAGE_LENGTH {
			return Err(format!("Message exceeds maximum length of {} bytes", MAX_MARKER_MESSAGE_LENGTH));
		}

		// Validate position consistency
		if StartLineNumber > EndLineNumber {
			return Err("Start line number cannot be greater than end line number".to_string());
		}

		// Validate column consistency within same line
		if StartLineNumber == EndLineNumber && StartColumn > EndColumn {
			return Err("Start column cannot be greater than end column on the same line".to_string());
		}

		Ok(Self {
			Severity,
			Message,
			StartLineNumber,
			StartColumn,
			EndLineNumber,
			EndColumn,
			Source:None,
			Code:None,
			ModelVersionId:None,
			RelatedInformation:None,
			Tags:None,
		})
	}
