// File: Mountain/Source/Environment/ConfigurationProvider.rs
// Role: Implements the `ConfigurationProvider` and `ConfigurationInspector`
// traits. Responsibilities:
//   - Core logic for reading, merging, updating, and inspecting settings.
//   - Handles the configuration cascade (Default -> User -> WorkSpace).
//   - Interacts with the file system via effects for persistence.

//! # ConfigurationProvider Implementation
//!
//! Implements the `ConfigurationProvider` and `ConfigurationInspector` traits
//! for the `MountainEnvironment`. This provider contains the core logic for
//! configuration management, including reading, merging, updating, and
//! inspecting settings from various sources.

use std::{path::PathBuf, sync::Arc};

use Common::{
	Configuration::{
		ConfigurationInspector::ConfigurationInspector,
		ConfigurationProvider::ConfigurationProvider,
		DTO::{
			ConfigurationOverridesDTO::ConfigurationOverridesDTO,
			ConfigurationTarget::ConfigurationTarget,
			InspectResultDataDTO::InspectResultDataDTO,
		},
	},
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::{ReadFile::ReadFile, WriteFileBytes::WriteFileBytes},
};
use async_trait::async_trait;
use log::{debug, info};
use serde_json::{Map, Value};
use tauri::Manager;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::{
	ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO,
	ExtensionManagement::Scanner::CollectDefaultConfigurations,
	RunTime::ApplicationRunTime::ApplicationRunTime as MountainRunTime,
};

#[async_trait]
impl ConfigurationProvider for MountainEnvironment {
	/// Retrieves a configuration value from the cached, merged configuration.
	async fn GetConfigurationValue(
		&self,
		Section:Option<String>,
		_Overrides:ConfigurationOverridesDTO,
	) -> Result<Value, CommonError> {
		debug!("[ConfigurationProvider] Getting configuration for section: {:?}", Section);
		let ConfigurationGuard = self
			.ApplicationState
			.Configuration
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		Ok(ConfigurationGuard.GetValue(Section.as_deref()))
	}

	/// Updates a configuration value in the appropriate `settings.json` file.
	async fn UpdateConfigurationValue(
		&self,
		Key:String,
		ValueToSet:Value,
		Target:ConfigurationTarget,
		_Overrides:ConfigurationOverridesDTO,
		_ScopeToLanguage:Option<bool>,
	) -> Result<(), CommonError> {
		info!("[ConfigurationProvider] Updating key '{}' in target {:?}", Key, Target);

		let RunTime = self.ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();

		let ConfigPath:PathBuf = match Target {
			ConfigurationTarget::User => {
				self.ApplicationHandle
					.path()
					.app_config_dir()
					.map(|p| p.join("settings.json"))
					.map_err(|e| {
						CommonError::ConfigurationLoad {
							Description:format!("Could not resolve user config path: {}", e),
						}
					})?
			},
			ConfigurationTarget::WorkSpace => {
				self.ApplicationState
					.WorkSpaceConfigurationPath
					.lock()
					.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
					.clone()
					.ok_or_else(|| {
						CommonError::ConfigurationLoad { Description:"No workspace configuration path set".into() }
					})?
			},
			_ => {
				return Err(CommonError::NotImplemented {
					FeatureName:"This configuration target is not supported".into(),
				});
			},
		};

		// Read the file, modify it, and write it back.
		let Bytes = RunTime.Run(ReadFile(ConfigPath.clone())).await.unwrap_or_default();
		let mut CurrentConfig:Value = serde_json::from_slice(&Bytes).unwrap_or_else(|_| Value::Object(Map::new()));

		if let Value::Object(Map) = &mut CurrentConfig {
			if ValueToSet.is_null() {
				Map.remove(&Key);
			} else {
				Map.insert(Key.clone(), ValueToSet);
			}
		}

		let ContentBytes = serde_json::to_vec_pretty(&CurrentConfig)
			.map_err(|e| CommonError::SerializationError { Description:e.to_string() })?;

		RunTime
			.Run(WriteFileBytes(ConfigPath.clone(), ContentBytes, true, true))
			.await?;

		// Re-merge all configurations to update the live state.
		InitializeAndMergeConfigurations(self).await?;
		Ok(())
	}
}

#[async_trait]
impl ConfigurationInspector for MountainEnvironment {
	/// Inspects a configuration key to get its value from all relevant scopes.
	async fn InspectConfigurationValue(
		&self,
		Key:String,
		_Overrides:ConfigurationOverridesDTO,
	) -> Result<Option<InspectResultDataDTO>, CommonError> {
		info!("[ConfigurationProvider] Inspecting key: {}", Key);

		let UserSettingsPath = self
			.ApplicationHandle
			.path()
			.app_config_dir()
			.map(|p| p.join("settings.json"))
			.ok();
		let WorkSpaceSettingsPath = self.ApplicationState.WorkSpaceConfigurationPath.lock().unwrap().clone();

		// Read each configuration layer individually.
		let DefaultConfig = CollectDefaultConfigurations(&self.ApplicationState)?;
		let UserConfig = ReadAndParseConfigurationFile(self, &UserSettingsPath).await?;
		let WorkSpaceConfig = ReadAndParseConfigurationFile(self, &WorkSpaceSettingsPath).await?;

		let GetValueFromDotPath =
			|Node:&Value, Path:&str| -> Option<Value> { Path.split('.').try_fold(Node, |n, key| n.get(key)).cloned() };

		let mut ResultDTO:InspectResultDataDTO = InspectResultDataDTO::default();
		ResultDTO.DefaultValue = GetValueFromDotPath(&DefaultConfig, &Key);
		ResultDTO.UserValue = GetValueFromDotPath(&UserConfig, &Key);
		ResultDTO.WorkSpaceValue = GetValueFromDotPath(&WorkSpaceConfig, &Key);

		// Determine the final effective value based on the correct cascade order.
		ResultDTO.EffectiveValue = ResultDTO
			.WorkSpaceValue
			.clone()
			.or_else(|| ResultDTO.UserValue.clone())
			.or_else(|| ResultDTO.DefaultValue.clone());

		if ResultDTO.EffectiveValue.is_some() { Ok(Some(ResultDTO)) } else { Ok(None) }
	}
}

/// An internal helper to read and parse a single JSON configuration file.
async fn ReadAndParseConfigurationFile(
	Environment:&MountainEnvironment,
	Path:&Option<PathBuf>,
) -> Result<Value, CommonError> {
	if let Some(p) = Path {
		let RunTime = Environment.ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();
		if let Ok(Bytes) = RunTime.Run(ReadFile(p.clone())).await {
			return Ok(serde_json::from_slice(&Bytes).unwrap_or(Value::Object(Map::new())));
		}
	}
	Ok(Value::Object(Map::new()))
}

/// Logic to load and merge all configuration files into the effective
/// configuration stored in `ApplicationState`.
pub async fn InitializeAndMergeConfigurations(Environment:&MountainEnvironment) -> Result<(), CommonError> {
	info!("[ConfigurationProvider] Re-initializing and merging all configurations...");

	let DefaultConfig = CollectDefaultConfigurations(&Environment.ApplicationState)?;
	let UserSettingsPath = Environment
		.ApplicationHandle
		.path()
		.app_config_dir()
		.map(|p| p.join("settings.json"))
		.ok();
	let WorkSpaceSettingsPath = Environment.ApplicationState.WorkSpaceConfigurationPath.lock().unwrap().clone();

	let UserConfig = ReadAndParseConfigurationFile(Environment, &UserSettingsPath).await?;
	let WorkSpaceConfig = ReadAndParseConfigurationFile(Environment, &WorkSpaceSettingsPath).await?;

	// A true deep merge is required here.
	let mut Merged = DefaultConfig.as_object().cloned().unwrap_or_default();
	if let Some(UserMap) = UserConfig.as_object() {
		for (k, v) in UserMap {
			Merged.insert(k.clone(), v.clone());
		}
	}
	if let Some(WorkSpaceMap) = WorkSpaceConfig.as_object() {
		for (k, v) in WorkSpaceMap {
			Merged.insert(k.clone(), v.clone());
		}
	}

	let FinalConfig = MergedConfigurationStateDTO::Create(Value::Object(Merged));
	*Environment.ApplicationState.Configuration.lock().unwrap() = FinalConfig;

	info!("[ConfigurationProvider] Configuration state updated and merged.");
	Ok(())
}
