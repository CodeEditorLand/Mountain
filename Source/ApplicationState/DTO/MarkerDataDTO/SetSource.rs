//! `MarkerDataDTO::SetSource`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use super::MarkerSeverity::MarkerSeverity;

pub fn Fn(This:&mut Struct, Source:String) -> Result<(), String> {
		if Source.len() > MAX_SOURCE_LENGTH {
			return Err(format!("Source exceeds maximum length of {} bytes", MAX_SOURCE_LENGTH));
		}

		This.Source = Some(Source);

		Ok(())
	}
