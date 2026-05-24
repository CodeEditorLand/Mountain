//! `MergedConfigurationStateDTO::GetNumber`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::dev_log;

pub fn Fn(This:&Struct, Section:&str, Default:f64) -> f64 {
		This.GetValue(Some(Section)).as_f64().unwrap_or(Default)
	}
