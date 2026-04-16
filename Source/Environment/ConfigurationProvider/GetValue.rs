//! Configuration value retrieval.

use CommonLibrary::{
	Configuration::DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
	Error::CommonError::CommonError,
};
use serde_json::Value;
use crate::dev_log;

/// Retrieves a configuration value from the cached, merged configuration.
pub(super) async fn get_configuration_value(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,
	section:Option<String>,
	_overrides:ConfigurationOverridesDTO,
) -> Result<Value, CommonError> {
	dev_log!("config", "[ConfigurationProvider] Getting configuration for section: {:?}", section);

	let configuration_guard = environment
		.ApplicationState
		.Configuration
		.GlobalConfiguration
		.lock()
		.map_err(|e| CommonError::StateLockPoisoned { Context:format!("Failed to lock configuration: {}", e) })?;

	let configuration_value = match section.as_deref() {
		None => (*configuration_guard).clone(),
		Some(section_path) => {
			// Navigate through the configuration using dot notation
			let mut current = &*configuration_guard;
			for key in section_path.split('.') {
				current = match current.get(key) {
					Some(value) => value,
					None => {
						dev_log!("config", "warn: [ConfigurationProvider] Configuration section '{}' not found in path: {:?}",
							key, section_path);
						return Ok(Value::Null);
					},
				};
			}
			current.clone()
		},
	};

	// Validate that the configuration value exists
	if configuration_value.is_null() {
		dev_log!("config", "warn: [ConfigurationProvider] Configuration section not found: {:?}", section);
	}

	Ok(configuration_value)
}
