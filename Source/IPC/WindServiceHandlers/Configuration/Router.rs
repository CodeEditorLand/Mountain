//! Configuration command router.
//!
//! Routes all `configuration:*`, `environment:get`, and
//! `workbench:getConfiguration` commands to their handlers.

use std::sync::Arc;

use serde_json::{Value, json};
use tauri::Emitter;
use CommonLibrary::{
	Configuration::{
		ConfigurationInspector::ConfigurationInspector,
		DTO::ConfigurationOverridesDTO::ConfigurationOverridesDTO,
	},
	Environment::Requires::Requires,
};

use crate::{
	IPC::WindServiceHandlers::Configuration::{
		EnvironmentGet::Fn as EnvironmentGet,
		Get::Fn as ConfigurationGet,
		Update::Fn as ConfigurationUpdate,
		Workbench::Fn as WorkbenchConfiguration,
	},
	RunTime::ApplicationRunTime::ApplicationRunTime,
	dev_log,
};

/// Routes configuration, environment, and workbench-configuration commands.
/// Returns `Some(result)` for handled commands, `None` otherwise.
pub(crate) async fn route(
	RunTime:Arc<ApplicationRunTime>,

	ApplicationHandle:tauri::AppHandle,

	command:&str,

	Arguments:Vec<Value>,
) -> Option<Result<Value, String>> {
	match command {
		// --- configuration:get family ---
		"configuration:get" | "configuration:getValue" | "configuration:lookup" => {
			Some(ConfigurationGet(RunTime, Arguments).await)
		},

		// --- configuration:update family ---
		"configuration:update" | "configuration:updateValue" => {
			let result = ConfigurationUpdate(RunTime, Arguments).await;

			// On successful update, broadcast the change to Sky so
			// the workbench theme/settings UI reflects the new value
			// without a full reload.
			if result.is_ok() {
				let _ = ApplicationHandle.emit("sky://configuration/changed", json!({}));
			}

			Some(result)
		},

		// `ConfigurationService` listens for `onDidChange` from
		// the channel on the binary IPC rail. Mountain broadcasts
		// config changes via a Tauri event directly; ack the
		// channel-listen with Null so the ChannelClient doesn't
		// leak a pending promise.
		"configuration:onDidChange" => Some(Ok(Value::Null)),

		// `configuration:inspect` is `IConfigurationService.inspect(key)`.
		// VS Code destructures `{ value, defaultValue, userValue,
		// workspaceValue, workspaceFolderValue, memoryValue }` from
		// the result. `InspectConfigurationValue` reads each scope
		// individually so the Settings UI can show which scope is
		// overriding a given key.
		"configuration:inspect" => {
			dev_log!("config", "configuration:inspect");

			let Key = Arguments.get(0).and_then(|v| v.as_str()).unwrap_or("");

			let Inspector:Arc<dyn ConfigurationInspector> = RunTime.Environment.Require();

			let result = match Inspector
				.InspectConfigurationValue(Key.to_string(), ConfigurationOverridesDTO::default())
				.await
			{
				Ok(Some(Result)) => {
					Ok(json!({
						"value": Result.EffectiveValue,
						"defaultValue": Result.DefaultValue,
						"userValue": Result.UserValue,
						"workspaceValue": Result.WorkspaceValue,
						"workspaceFolderValue": Result.WorkspaceFolderValue,
						"memoryValue": Result.MemoryValue,
					}))
				},

				Ok(None) => {
					// Key not found in any scope - fall back to merged value
					// so the Settings UI gets `undefined` rather than crashing.
					let Fallback = ConfigurationGet(RunTime.clone(), Arguments.clone())
						.await
						.unwrap_or(Value::Null);

					Ok(json!({
						"value": Fallback,
						"defaultValue": Fallback,
						"userValue": Value::Null,
						"workspaceValue": Value::Null,
						"workspaceFolderValue": Value::Null,
						"memoryValue": Value::Null,
					}))
				},

				Err(Error) => {
					dev_log!("config", "warn: configuration:inspect error for '{}': {}", Key, Error);

					Ok(json!({
						"value": Value::Null,
						"defaultValue": Value::Null,
						"userValue": Value::Null,
						"workspaceValue": Value::Null,
						"workspaceFolderValue": Value::Null,
						"memoryValue": Value::Null,
					}))
				},
			};

			Some(result)
		},

		// --- environment:get ---
		"environment:get" => Some(EnvironmentGet(RunTime, Arguments).await),

		// --- workbench:getConfiguration ---
		"workbench:getConfiguration" => Some(WorkbenchConfiguration(RunTime, Arguments).await),

		_ => None,
	}
}
