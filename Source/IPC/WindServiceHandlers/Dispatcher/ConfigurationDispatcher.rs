//! Configuration, environment, and workbench-configuration command dispatcher.

use CommonLibrary::Configuration::DTO::{
	ConfigurationOverridesDTO as ConfigurationOverridesDTOModule,
	ConfigurationTarget as ConfigurationTargetModule,
};
use serde_json::{Value, json};

<<<<<<< HEAD
use crate::Configuration::{
=======
use crate::IPC::WindServiceHandlers::Configuration::{
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
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
<<<<<<< HEAD
	runtime:&crate::RunTime::ApplicationRunTime::ApplicationRunTime,
=======
	runtime:std::sync::Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867

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
<<<<<<< HEAD
			// Return the per-scope breakdown VS Code expects.
			// Field names match IConfigurationService.inspect() result shape:
			// value / defaultValue / userValue / workspaceValue / workspaceFolderValue /
			// memoryValue.
			let current_value = ConfigurationGet(runtime.clone(), arguments.clone())
				.await
				.unwrap_or(Value::Null);

			Ok(json!({
				"value": current_value,
				"defaultValue": current_value,
				"userValue": Value::Null,
				"workspaceValue": Value::Null,
				"workspaceFolderValue": Value::Null,
				"memoryValue": Value::Null,
=======
			let current_value = ConfigurationGet(runtime.clone(), arguments).await.unwrap_or(Value::Null);

			Ok(json!({
				"value": current_value,
				"default": current_value,
				"user": Value::Null,
				"workspace": Value::Null,
				"workspaceFolder": Value::Null,
				"memory": Value::Null,
>>>>>>> 8e05e904fef6242d1b7fe4804dd9ac660dc91867
			}))
		},

		"environment:get" => EnvironmentGet(runtime.clone(), arguments).await,

		"workbench:getConfiguration" => WorkbenchConfiguration(runtime.clone(), arguments).await,

		_ => Err(format!("Unknown configuration command: {}", command)),
	}
}
