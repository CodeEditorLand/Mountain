//! `MergedConfigurationStateDTO::GetBool`

use super::Struct;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::dev_log;

pub fn Fn(This:&Struct, Section:&str, Default:bool) -> bool {
		This.GetValue(Some(Section)).as_bool().unwrap_or(Default)
	}
