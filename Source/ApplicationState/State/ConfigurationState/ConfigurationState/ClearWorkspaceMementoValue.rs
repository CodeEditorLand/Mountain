//! `ConfigurationState::ClearWorkspaceMementoValue`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

pub fn Fn(This:&Struct, key:&str) {
		if let Ok(mut guard) = This.MementoWorkspaceStorage.lock() {
			guard.remove(key);

			dev_log!(
				"config",
				"[ConfigurationState] Workspace memento value removed for key: {}",
				key
			);
		}
	}
