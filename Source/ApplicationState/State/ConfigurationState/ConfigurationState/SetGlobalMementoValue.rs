//! `ConfigurationState::SetGlobalMementoValue`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

pub fn Fn(This:&Struct, key:String, value:serde_json::Value) {
		if let Ok(mut guard) = This.MementoGlobalStorage.lock() {
			guard.insert(key.clone(), value);

			dev_log!("config", "[ConfigurationState] Global memento value updated for key: {}", key);
		}
	}
