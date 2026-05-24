//! `MergedConfigurationStateDTO::GetString`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::dev_log;

pub fn Fn(This:&Struct, Section:&str, Default:&str) -> String {
		This.GetValue(Some(Section)).as_str().unwrap_or(Default).to_string()
	}
