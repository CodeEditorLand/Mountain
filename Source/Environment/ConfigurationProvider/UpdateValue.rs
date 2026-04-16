//! Configuration value update and persistence.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	Configuration::DTO::{
		ConfigurationOverridesDTO::ConfigurationOverridesDTO,
		ConfigurationTarget::ConfigurationTarget,
	},
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::{ReadFile::ReadFile, WriteFileBytes::WriteFileBytes},
};
use serde_json::{Map, Value};
use tauri::Manager;

use crate::{Environment::Utility, RunTime::ApplicationRunTime::RuntimeStruct::ApplicationRunTime};
use crate::dev_log;

/// Updates a configuration value in the appropriate `settings.json` file.
pub(super) async fn update_configuration_value(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,
	key:String,
	value:Value,
	target:ConfigurationTarget,
	_overrides:ConfigurationOverridesDTO,
	_scope_to_language:Option<bool>,
) -> Result<(), CommonError> {
	dev_log!("config", "[ConfigurationProvider] Updating key '{}' in target {:?}", key, target);

	let runtime = environment.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let config_path:PathBuf = match target {
		ConfigurationTarget::User => {
			environment
				.ApplicationHandle
				.path()
				.app_config_dir()
				.map(|p| p.join("settings.json"))
				.map_err(|error| {
					CommonError::ConfigurationLoad {
						Description:format!("Could not resolve user config path: {}", error),
					}
				})?
		},

		ConfigurationTarget::Workspace => {
			environment
				.ApplicationState
				.Workspace
				.WorkspaceConfigurationPath
				.lock()
				.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
				.clone()
				.ok_or_else(|| {
					CommonError::ConfigurationLoad { Description:"No workspace configuration path set".into() }
				})?
		},

		_ => {
			dev_log!("config", "warn: [ConfigurationProvider] Unsupported configuration target: {:?}", target);

			return Err(CommonError::NotImplemented {
				FeatureName:"This configuration target is not supported".into(),
			});
		},
	};

	// Read the file, modify it, and write it back.
	let bytes = runtime.Run(ReadFile(config_path.clone())).await.unwrap_or_default();

	let mut current_config:Value = serde_json::from_slice(&bytes).unwrap_or_else(|_| Value::Object(Map::new()));

	if let Value::Object(map) = &mut current_config {
		if value.is_null() {
			map.remove(&key);
			dev_log!("config", "[ConfigurationProvider] Removed configuration key '{}'", key);
		} else {
			map.insert(key.clone(), value.clone());
			dev_log!("config", "[ConfigurationProvider] Updated configuration key '{}'", key);
		}
	}

	let content_bytes = serde_json::to_vec_pretty(&current_config)?;

	runtime
		.Run(WriteFileBytes(config_path.clone(), content_bytes, true, true))
		.await?;

	// Re-merge all configurations to update the live state.
	crate::Environment::ConfigurationProvider::Loading::initialize_and_merge_configurations(environment).await?;

	Ok(())
}
