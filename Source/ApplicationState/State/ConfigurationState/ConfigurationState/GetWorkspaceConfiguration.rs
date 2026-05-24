//! `ConfigurationState::GetWorkspaceConfiguration`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

pub fn Fn(This:&Struct) -> serde_json::Value {
		This.WorkspaceConfiguration
			.lock()
			.map(|g| g.clone())
			.unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
	}
