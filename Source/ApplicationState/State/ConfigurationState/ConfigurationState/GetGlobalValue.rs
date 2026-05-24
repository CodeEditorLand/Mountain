//! `ConfigurationState::GetGlobalValue`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

pub fn Fn(This:&Struct, path:&str) -> Option<serde_json::Value> {
		This.GetGlobalConfiguration().Get(path).cloned()
	}
