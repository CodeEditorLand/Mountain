//! Configuration, environment, and workbench-configuration handlers.

use std::sync::Arc;

use serde_json::{Value, json};
use CommonLibrary::Configuration::DTO::{
	ConfigurationOverridesDTO as ConfigurationOverridesDTOModule,
	ConfigurationTarget as ConfigurationTargetModule,
};

type ConfigurationOverridesDTO = ConfigurationOverridesDTOModule::ConfigurationOverridesDTO;

type ConfigurationTarget = ConfigurationTargetModule::ConfigurationTarget;

use CommonLibrary::{Configuration::ConfigurationProvider::ConfigurationProvider, Environment::Requires::Requires};

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Handler for configuration get requests
pub async fn ConfigurationGet(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let key = Arguments
		.get(0)
		.ok_or("Missing configuration key".to_string())?
		.as_str()
		.ok_or("Configuration key must be a string".to_string())?;

	let provider:Arc<dyn ConfigurationProvider> = RunTime.Environment.Require();

	let value = provider
		.GetConfigurationValue(Some(key.to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|Error| format!("Failed to get configuration: {}", Error))?;

	dev_log!("config", "get: {} = {:?}", key, value);

	Ok(value)
}

/// Handler for configuration update requests
pub async fn ConfigurationUpdate(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let key = Arguments
		.get(0)
		.ok_or("Missing configuration key".to_string())?
		.as_str()
		.ok_or("Configuration key must be a string".to_string())?;

	let value = Arguments.get(1).ok_or("Missing configuration value".to_string())?.clone();

	let provider:Arc<dyn ConfigurationProvider> = RunTime.Environment.Require();

	provider
		.UpdateConfigurationValue(
			key.to_string(),
			value,
			ConfigurationTarget::User,
			ConfigurationOverridesDTO::default(),
			None,
		)
		.await
		.map_err(|Error| format!("Failed to update configuration: {}", Error))?;

	dev_log!("config", "updated: {}", key);

	Ok(Value::Null)
}

/// Handler for workbench configuration requests
pub async fn WorkbenchConfiguration(RunTime:Arc<ApplicationRunTime>, _Arguments:Vec<Value>) -> Result<Value, String> {
	let provider:Arc<dyn ConfigurationProvider> = RunTime.Environment.Require();

	let config = provider
		.GetConfigurationValue(None, ConfigurationOverridesDTO::default())
		.await
		.map_err(|Error| format!("Failed to get workbench configuration: {}", Error))?;

	dev_log!("config", "workbench config retrieved");

	Ok(config)
}

/// Handler for environment get requests
pub async fn EnvironmentGet(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	let key = Arguments
		.get(0)
		.ok_or("Missing environment key".to_string())?
		.as_str()
		.ok_or("Environment key must be a string".to_string())?;

	let value = std::env::var(key).map_err(|Error| format!("Failed to get environment variable: {}", Error))?;

	dev_log!("config", "env_get: {}", key);

	Ok(json!(value))
}
