//! `ConfigurationState::ClearGlobalMemento`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

pub fn Fn(This:&Struct) {
		if let Ok(mut guard) = This.MementoGlobalStorage.lock() {
			guard.clear();

			dev_log!("config", "[ConfigurationState] Global memento storage cleared");
		}
	}
