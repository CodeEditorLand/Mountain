//! Wire method: `workbench:getConfigurationValue`.

use std::sync::Arc;

use serde_json::Value;

use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

pub async fn Fn(RunTime:Arc<ApplicationRunTime>, _Arguments:Vec<Value>) -> Result<Value, String> {
	use CommonLibrary::{
		Configuration::{
			ConfigurationProvider::ConfigurationProvider,
			DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
		},
		Environment::Requires::Requires,
	};

	let provider:Arc<dyn ConfigurationProvider> = RunTime.Environment.Require();

	let config = provider
		.GetConfigurationValue(None, ConfigurationOverridesDTO::default())
		.await
		.map_err(|Error| format!("Failed to get workbench configuration: {}", Error))?;

	dev_log!("config", "workbench config retrieved");

	Ok(config)
}
