//! Configuration loading and merging utilities.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::ReadFile::ReadFile,
};
use serde_json::{Map, Value};
use tauri::Manager;

use crate::{
use crate::dev_log;
	ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO,
	Environment::Utility,
	RunTime::ApplicationRunTime::RuntimeStruct::ApplicationRunTime,
};

/// An internal helper to read and parse a single JSON configuration file.
pub(super) async fn read_and_parse_configuration_file(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,
	path:&Option<PathBuf>,
) -> Result<Value, CommonError> {
	if let Some(p) = path {
		let runtime = environment.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

		if let Ok(bytes) = runtime.Run(ReadFile(p.clone())).await {
			return Ok(serde_json::from_slice(&bytes).unwrap_or_else(|_| Value::Object(Map::new())));
		}
	}

	Ok(Value::Object(Map::new()))
}

/// Logic to load and merge all configuration files into the effective
/// configuration stored in `ApplicationState`.
pub async fn initialize_and_merge_configurations(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,
) -> Result<(), CommonError> {
	dev_log!("config", "[ConfigurationProvider] Re-initializing and merging all configurations...");

	let default_config = collect_default_configurations(&environment.ApplicationState)?;

	let user_settings_path = environment
		.ApplicationHandle
		.path()
		.app_config_dir()
		.map(|p| p.join("settings.json"))
		.ok();

	let workspace_settings_path = environment
		.ApplicationState
		.Workspace
		.WorkspaceConfigurationPath
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.clone();

	let user_config = read_and_parse_configuration_file(environment, &user_settings_path).await?;

	let workspace_config = read_and_parse_configuration_file(environment, &workspace_settings_path).await?;

	// A true deep merge is required here. The merge order matches the cascade:
	// Default (base) → User (overrides default) → Workspace (overrides user)
	let mut merged = default_config.as_object().cloned().unwrap_or_default();

	if let Some(user_map) = user_config.as_object() {
		for (key, value) in user_map {
			// Deep merge nested objects, shallow merge at root level
			if value.is_object() && merged.get(key.as_str()).is_some_and(|v| v.is_object()) {
				if let (Some(user_value), Some(_base_value)) =
					(value.as_object(), merged.get(key.as_str()).and_then(|v| v.as_object()))
				{
					for (inner_key, inner_value) in user_value {
						merged.get_mut(key.as_str()).and_then(|v| v.as_object_mut()).map(|m| {
							m.insert(inner_key.clone(), inner_value.clone());
						});
					}
				}
			} else {
				merged.insert(key.clone(), value.clone());
			}
		}
	}

	if let Some(workspace_map) = workspace_config.as_object() {
		for (key, value) in workspace_map {
			if value.is_object() && merged.get(key.as_str()).is_some_and(|v| v.is_object()) {
				if let (Some(workspace_value), Some(_base_value)) =
					(value.as_object(), merged.get(key.as_str()).and_then(|v| v.as_object()))
				{
					for (inner_key, inner_value) in workspace_value {
						merged.get_mut(key.as_str()).and_then(|v| v.as_object_mut()).map(|m| {
							m.insert(inner_key.clone(), inner_value.clone());
						});
					}
				}
			} else {
				merged.insert(key.clone(), value.clone());
			}
		}
	}

	let configuration_size = merged.len();
	let final_config = MergedConfigurationStateDTO::Create(Value::Object(merged));

	*environment
		.ApplicationState
		.Configuration
		.GlobalConfiguration
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)? = final_config.Data;

	dev_log!("config", 
		"[ConfigurationProvider] Configuration merged successfully with {} top-level keys.",
		configuration_size
	);

	Ok(())
}

/// Collects default configurations from all installed extensions.
pub(super) fn collect_default_configurations(
	application_state:&crate::ApplicationState::ApplicationState,
) -> Result<Value, CommonError> {
	let mut default_config = Map::new();

	// Collect configurations from all extensions' contributes.configuration
	for extension in application_state
		.Extension
		.ScannedExtensions
		.ScannedExtensions
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.values()
	{
		if let Some(contributes) = &extension.Contributes {
			if let Some(config_array) = contributes.get("configuration").and_then(|c| c.as_array()) {
				for config_value in config_array {
					// Each config contribution may have "key" and "value"
					if let (Some(key), Some(value)) =
						(config_value.get("key").and_then(|k| k.as_str()), config_value.get("value"))
					{
						if let Some(value_obj) = value.as_object() {
							default_config.insert(key.to_string(), Value::Object(value_obj.clone()));
						}
					}
				}
			}
		}
	}

	Ok(Value::Object(default_config))
}
