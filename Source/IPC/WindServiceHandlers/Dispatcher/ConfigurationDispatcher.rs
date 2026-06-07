//! Configuration, environment, and workbench-configuration command dispatcher.

use CommonLibrary::Configuration::DTO::{
	ConfigurationOverridesDTO as ConfigurationOverridesDTOModule,
	ConfigurationTarget as ConfigurationTargetModule,
};
use serde_json::{Value, json};

use crate::Configuration::{
	EnvironmentGet::Fn as EnvironmentGet,
	Get::Fn as ConfigurationGet,
	Update::Fn as ConfigurationUpdate,
	Workbench::Fn as WorkbenchConfiguration,
};

type ConfigurationOverridesDTO = ConfigurationOverridesDTOModule::ConfigurationOverridesDTO;

type ConfigurationTarget = ConfigurationTargetModule::ConfigurationTarget;

/// Dispatches configuration-related commands.
///
/// Handled commands:
/// - `configuration:get` / `configuration:getValue`
/// - `configuration:update` / `configuration:updateValue`
/// - `configuration:onDidChange` (stub)
/// - `configuration:lookup` (alias to get)
/// - `configuration:inspect`
/// - `environment:get`
/// - `workbench:getConfiguration`
pub async fn dispatch_configuration(
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,

	app_handle:&tauri::AppHandle,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"configuration:get" | "configuration:getValue" => ConfigurationGet(runtime.clone(), arguments).await,

		"configuration:update" | "configuration:updateValue" => ConfigurationUpdate(runtime.clone(), arguments).await,

		"configuration:onDidChange" => Ok(Value::Null),

		"configuration:lookup" => ConfigurationGet(runtime.clone(), arguments).await,

		"configuration:inspect" => {
			let current_value = ConfigurationGet(runtime.clone(), arguments).await.unwrap_or(Value::Null);

			Ok(json!({
				"value": current_value,
				"default": current_value,
				"user": Value::Null,
				"workspace": Value::Null,
				"workspaceFolder": Value::Null,
				"memory": Value::Null,
			}))
		},

		"environment:get" => EnvironmentGet(runtime.clone(), arguments).await,

		"workbench:getConfiguration" => WorkbenchConfiguration(runtime.clone(), arguments).await,

		_ => Err(format!("Unknown configuration command: {}", command)),
	}
}
