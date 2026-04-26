#![allow(non_snake_case, unused_variables, dead_code, unused_imports)]

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
pub async fn handle_configuration_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing configuration key".to_string())?
		.as_str()
		.ok_or("Configuration key must be a string".to_string())?;

	let provider:Arc<dyn ConfigurationProvider> = runtime.Environment.Require();

	let value = provider
		.GetConfigurationValue(Some(key.to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|e| format!("Failed to get configuration: {}", e))?;

	dev_log!("config", "get: {} = {:?}", key, value);
	Ok(value)
}

/// Handler for configuration update requests
pub async fn handle_configuration_update(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing configuration key".to_string())?
		.as_str()
		.ok_or("Configuration key must be a string".to_string())?;

	let value = args.get(1).ok_or("Missing configuration value".to_string())?.clone();

	let provider:Arc<dyn ConfigurationProvider> = runtime.Environment.Require();

	provider
		.UpdateConfigurationValue(
			key.to_string(),
			value,
			ConfigurationTarget::User,
			ConfigurationOverridesDTO::default(),
			None,
		)
		.await
		.map_err(|e| format!("Failed to update configuration: {}", e))?;

	dev_log!("config", "updated: {}", key);
	Ok(Value::Null)
}

/// Handler for workbench configuration requests
pub async fn handle_workbench_configuration(
	runtime:Arc<ApplicationRunTime>,
	_args:Vec<Value>,
) -> Result<Value, String> {
	let provider:Arc<dyn ConfigurationProvider> = runtime.Environment.Require();

	let config = provider
		.GetConfigurationValue(None, ConfigurationOverridesDTO::default())
		.await
		.map_err(|e| format!("Failed to get workbench configuration: {}", e))?;

	dev_log!("config", "workbench config retrieved");
	Ok(config)
}

/// Handler for environment get requests
pub async fn handle_environment_get(runtime:Arc<ApplicationRunTime>, args:Vec<Value>) -> Result<Value, String> {
	let key = args
		.get(0)
		.ok_or("Missing environment key".to_string())?
		.as_str()
		.ok_or("Environment key must be a string".to_string())?;

	let value = std::env::var(key).map_err(|e| format!("Failed to get environment variable: {}", e))?;

	dev_log!("config", "env_get: {}", key);
	Ok(json!(value))
}
