#![allow(non_snake_case)]

//! Configuration domain handlers for Wind IPC.

use std::sync::Arc;

use serde_json::Value;

use CommonLibrary::{
	Configuration::ConfigurationProvider::ConfigurationProvider,
	Environment::Requires::Requires,
};

use crate::{
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

use super::{ConfigurationOverridesDTO, ConfigurationTarget};

/// Handler for configuration get requests
pub async fn handle_configuration_get(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Key = Args
		.get(0)
		.ok_or("Missing configuration key".to_string())?
		.as_str()
		.ok_or("Configuration key must be a string".to_string())?;

	let Provider:Arc<dyn ConfigurationProvider> = Runtime.Environment.Require();

	let ConfigValue = Provider
		.GetConfigurationValue(Some(Key.to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|E| format!("Failed to get configuration: {}", E))?;

	dev_log!("config", "get: {} = {:?}", Key, ConfigValue);
	Ok(ConfigValue)
}

/// Handler for configuration update requests
pub async fn handle_configuration_update(Runtime:Arc<ApplicationRunTime>, Args:Vec<Value>) -> Result<Value, String> {
	let Key = Args
		.get(0)
		.ok_or("Missing configuration key".to_string())?
		.as_str()
		.ok_or("Configuration key must be a string".to_string())?;

	let ConfigValue = Args.get(1).ok_or("Missing configuration value".to_string())?.clone();

	let Provider:Arc<dyn ConfigurationProvider> = Runtime.Environment.Require();

	Provider
		.UpdateConfigurationValue(
			Key.to_string(),
			ConfigValue,
			ConfigurationTarget::User,
			ConfigurationOverridesDTO::default(),
			None,
		)
		.await
		.map_err(|E| format!("Failed to update configuration: {}", E))?;

	dev_log!("config", "updated: {}", Key);
	Ok(Value::Null)
}

/// Handler for workbench configuration requests
pub async fn handle_workbench_configuration(Runtime:Arc<ApplicationRunTime>, _Args:Vec<Value>) -> Result<Value, String> {
	let Provider:Arc<dyn ConfigurationProvider> = Runtime.Environment.Require();

	let Config = Provider
		.GetConfigurationValue(None, ConfigurationOverridesDTO::default())
		.await
		.map_err(|E| format!("Failed to get workbench configuration: {}", E))?;

	dev_log!("config", "workbench config retrieved");
	Ok(Config)
}
