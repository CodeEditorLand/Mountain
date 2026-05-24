//! `MergedConfigurationStateDTO::GetValue`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::dev_log;

pub fn Fn(This:&Struct, Section:Option<&str>) -> Value {
		if let Some(Path) = Section {
			let Depth = Path.matches('.').count();

			if Depth > MAX_CONFIGURATION_DEPTH {
				dev_log!(
					"config",
					"warn: configuration path depth {} exceeds maximum of {}",
					Depth,
					MAX_CONFIGURATION_DEPTH
				);

				return Value::Null;
			}

			Path.split('.')
				.try_fold(&This.Data, |Node, Key| Node.get(Key))
				.unwrap_or(&Value::Null)
				.clone()
		} else {
			This.Data.clone()
		}
	}
