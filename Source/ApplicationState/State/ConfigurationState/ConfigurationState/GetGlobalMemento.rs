//! `ConfigurationState::GetGlobalMemento`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

pub fn Fn(This:&Struct) -> HashMap<String, serde_json::Value> {
		This.MementoGlobalStorage
			.lock()
			.ok()
			.map(|guard| guard.clone())
			.unwrap_or_default()
	}
