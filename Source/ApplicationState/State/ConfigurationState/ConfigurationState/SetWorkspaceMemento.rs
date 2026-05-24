//! `ConfigurationState::SetWorkspaceMemento`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

pub fn Fn(This:&Struct, storage:HashMap<String, serde_json::Value>) {
		if let Ok(mut guard) = This.MementoWorkspaceStorage.lock() {
			*guard = storage;
			dev_log!(
				"config",
				"[ConfigurationState] Workspace memento storage updated ({} keys)",
				guard.len()
			);
		}
	}
