//! Wire method: `configuration:get`.

use std::sync::Arc;

use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::{
		Configuration::{
			ConfigurationProvider::ConfigurationProvider,
			DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
		},
		Environment::Requires::Requires,
	};

	let Key = Arguments
		.Get(0)
		.ok_or("Missing configuration key".to_string())?
		.as_str()
		.ok_or("Configuration key must be a string".to_string())?;

	let Provider:Arc<dyn ConfigurationProvider> = RunTime.Environment.Require();

	let value = provider
		.GetConfigurationValue(Some(key.to_string()), ConfigurationOverridesDTO::default())
		.await
		.map_err(|Error| format!("Failed to get configuration: {}", Error))?;

	dev_log!("config", "get: {} = {:?}", key, value);

	Ok(value)
}
