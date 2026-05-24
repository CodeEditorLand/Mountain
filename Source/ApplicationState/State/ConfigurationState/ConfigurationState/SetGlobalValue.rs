//! `ConfigurationState::SetGlobalValue`

use super::Struct;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex as StandardMutex},
};
use crate::{ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO, dev_log};

pub fn Fn(This:&Struct, path:&str, value:serde_json::Value) {
		if let Ok(mut config_guard) = This.GlobalConfiguration.lock() {
			// Clone the current config for manipulation
			let current_config = (*config_guard).clone();

			// Create DTO to leverage its SetValue method
			let mut dto = MergedConfigurationStateDTO { Data:current_config };

			// Use the DTO's SetValue method which handles nested paths properly
			if let Err(e) = dto.SetValue(path, value) {
				dev_log!(
					"config",
					"warn: [ConfigurationState] Failed to set value at path '{}': {}",
					path,
					e
				);

				return;
			}

			// Write the updated data back
			*config_guard = dto.Data;

			dev_log!("config", "[ConfigurationState] Global configuration value updated at: {}", path);
		}
	}
