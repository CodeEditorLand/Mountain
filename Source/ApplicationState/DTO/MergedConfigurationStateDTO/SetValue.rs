//! `MergedConfigurationStateDTO::SetValue`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::dev_log;

pub fn Fn(This:&mut Struct, Section:&str, Value:Value) -> Result<(), String> {
		let Depth = Section.matches('.').count();

		if Depth > MAX_CONFIGURATION_DEPTH {
			return Err(format!(
				"Configuration path depth {} exceeds maximum of {}",
				Depth, MAX_CONFIGURATION_DEPTH
			));
		}

		let Keys:Vec<&str> = Section.split('.').collect();

		if Keys.is_empty() {
			return Err("Section path cannot be empty".to_string());
		}

		// Navigate or create nested structure
		let MutData = &mut This.Data;

		Struct::SetValueRecursive(MutData, &Keys, 0, Value);

		Ok(())
	}
