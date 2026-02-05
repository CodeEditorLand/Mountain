//! Configuration value retrieval.

use CommonLibrary::{
	Configuration::DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
	Error::CommonError::CommonError,
};
use log::{debug, warn};
use serde_json::Value;

use crate::Environment::Utility;

/// Retrieves a configuration value from the cached, merged configuration.
pub(super) async fn get_configuration_value(
	environment: &crate::Environment::MountainEnvironment::MountainEnvironment,
	section: Option<String>,
	_overrides: ConfigurationOverridesDTO,
) -> Result<Value, CommonError> {
	debug!("[ConfigurationProvider] Getting configuration for section: {:?}", section);

	let configuration_guard = environment
		.ApplicationState
		.Configuration
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

	let configuration_value = configuration_guard.GetValue(section.as_deref());

	// Validate that the configuration value exists
	if configuration_value.is_null() {
		warn!("[ConfigurationProvider] Configuration section not found: {:?}", section);
	}

	Ok(configuration_value)
}
