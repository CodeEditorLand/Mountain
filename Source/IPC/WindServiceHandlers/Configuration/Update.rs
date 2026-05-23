
//! Wire method: `configuration:update`.

use std::sync::Arc;

use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::{
		Configuration::{
			ConfigurationProvider::ConfigurationProvider,
			DTO::{ConfigurationOverridesDTO::ConfigurationOverridesDTO, ConfigurationTarget::ConfigurationTarget},
		},
		Environment::Requires::Requires,
	};

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
