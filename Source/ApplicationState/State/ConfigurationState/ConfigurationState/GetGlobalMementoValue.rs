//! `ConfigurationState::GetGlobalMementoValue`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

pub fn Fn(This:&Struct, key:&str) -> Option<serde_json::Value> {
		This.MementoGlobalStorage.lock().ok().and_then(|guard| guard.get(key).cloned())
	}
