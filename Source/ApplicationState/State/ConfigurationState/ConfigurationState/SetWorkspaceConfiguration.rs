//! `ConfigurationState::SetWorkspaceConfiguration`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

pub fn Fn(This:&Struct, config:serde_json::Value) {
		if let Ok(mut guard) = This.WorkspaceConfiguration.lock() {
			*guard = config;
			dev_log!("config", "[ConfigurationState] Workspace configuration updated");
		}
	}
