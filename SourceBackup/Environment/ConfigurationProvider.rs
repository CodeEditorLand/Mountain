// @module ConfigurationProvider (Environment)
// @description Implements the `ConfigProvider` and `ConfigInspector` traits for
// `MountainEnvironment`.

use std::sync::Arc;

use async_trait::async_trait;
use Common::{
	config::{
		ConfigInspector,
		ConfigProvider,
		DTO::{ConfigurationOverridesDTO, ConfigurationTarget, InspectResultDataDTO},
	},
	Environment::Requires,
	error::CommonError,
};
use serde_json::Value;

use super::MountainEnvironment;
use crate::Handler::config as ConfigHandler;

#[async_trait]
impl ConfigProvider for MountainEnvironment {
	// Retrieves a configuration value for a given section/key, applying
	// specified overrides.
	async fn GetConfigurationValue(
		&self,
		section:Option<String>,
		overrides:ConfigurationOverridesDTO,
	) -> Result<Value, CommonError> {
		ConfigHandler::GetConfigurationValueLogic(&self.ApplicationHandle, section, overrides).await
	}

	// Updates a configuration value at a specific key and target scope.
	async fn UpdateConfigurationValue(
		&self,
		key:String,
		value_to_set:Value,
		target:ConfigurationTarget,
		overrides:ConfigurationOverridesDTO,
		scope_to_language:Option<bool>,
	) -> Result<(), CommonError> {
		ConfigHandler::UpdateConfigurationValueLogic(
			&self.ApplicationHandle,
			key,
			value_to_set,
			target,
			overrides,
			scope_to_language,
		)
		.await
	}
}

#[async_trait]
impl ConfigInspector for MountainEnvironment {
	// Inspects a configuration key to get its value from all relevant scopes.
	async fn InspectConfigurationValue(
		&self,
		key:String,
		overrides:ConfigurationOverridesDTO,
	) -> Result<Option<InspectResultDataDTO>, CommonError> {
		ConfigHandler::InspectConfigurationValueLogic(&self.ApplicationHandle, key, overrides).await
	}
}

impl Requires<Arc<dyn ConfigProvider + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ConfigProvider + Send + Sync> { Arc::new(self.clone()) }
}

impl Requires<Arc<dyn ConfigInspector + Send + Sync>> for MountainEnvironment {
	fn Require(&self) -> Arc<dyn ConfigInspector + Send + Sync> { Arc::new(self.clone()) }
}
