//! Configuration, environment, and workbench-configuration command dispatcher.

use std::sync::Arc;

use tauri::Emitter;

use CommonLibrary::Configuration::DTO::{
	ConfigurationOverridesDTO as ConfigurationOverridesDTOModule,
	ConfigurationTarget as ConfigurationTargetModule,
};
use serde_json::{Value, json};

use crate::IPC::WindServiceHandlers::Configuration::{
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
	runtime:Arc<crate::RunTime::ApplicationRunTime::ApplicationRunTime>,

	app_handle:&tauri::AppHandle,

	command:&str,

	arguments:Vec<Value>,
) -> Result<Value, String> {
	match command {
		"configuration:get" | "configuration:getValue" => ConfigurationGet(runtime.clone(), arguments).await,

		"configuration:update" | "configuration:updateValue" => {
			let Result = ConfigurationUpdate(runtime.clone(), arguments).await;

			// Broadcast change to Sky on success.
			if Result.is_ok() {
				let _ = app_handle.emit("sky://configuration/changed", serde_json::json!({}));
			}

			Result
		},

		"configuration:onDidChange" => Ok(Value::Null),

		"configuration:lookup" => ConfigurationGet(runtime.clone(), arguments).await,

		"configuration:inspect" => {
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
			}))
		},

		"environment:get" => EnvironmentGet(runtime.clone(), arguments).await,

		"workbench:getConfiguration" => WorkbenchConfiguration(runtime.clone(), arguments).await,

		_ => Err(format!("Unknown configuration command: {}", command)),
	}
}
