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
	Environment::Requires::Requires,
	Error::CommonError::CommonError,
	FileSystem::{FileSystemReader::FileSystemReader, FileSystemWriter::FileSystemWriter},
};
use async_trait::async_trait;
use log::{debug, info, warn};
use serde_json::{Map, Value};
use tauri::Manager;

use super::MountainEnvironment::MountainEnvironment;

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
			.map_err(super::Utility::MapApplicationStateLockErrorToCommonError)?;

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

		let config_path_result:Result<Option<PathBuf>, CommonError> = match Target {
			ConfigurationTarget::User => {
				self.ApplicationHandle
					.path()
					.app_config_dir()
					.map(|p| Some(p.join("settings.json")))
					.map_err(|e| CommonError::ConfigurationLoad { Description:e.to_string() })
			},
			ConfigurationTarget::WorkSpace => {
				Ok(self
					.ApplicationState
					.WorkSpaceConfigurationPath
					.lock()
					.map_err(super::Utility::MapApplicationStateLockErrorToCommonError)?
					.clone())
			},
			_ => {
				return Err(CommonError::NotImplemented {
					FeatureName:"This configuration target is not supported".into(),
				});
			},
		};

		let ConfigPath = config_path_result?;

		if let Some(Path) = ConfigPath {
			let mut CurrentConfig = ReadAndParseConfigurationFile(self, &Some(Path.clone())).await;

			if let Value::Object(Map) = &mut CurrentConfig {
				if ValueToSet.is_null() {
					Map.remove(&Key);
				} else {
					Map.insert(Key.clone(), ValueToSet);
				}
			}

			let ContentBytes = serde_json::to_vec_pretty(&CurrentConfig)
				.map_err(|e| CommonError::SerializationError { Description:e.to_string() })?;

			let FileSystemWriter:Arc<dyn FileSystemWriter> = self.Require();
			FileSystemWriter.WriteFile(&Path, ContentBytes, true, true).await?;

			InitializeAndMergeConfigurations(self).await;
			Ok(())
		} else {
			Err(CommonError::ConfigurationUpdate {
				Key,
				Description:format!("Configuration target {:?} is not available.", Target),
			})
		}
	}
}

#[async_trait]
impl ConfigurationInspector for MountainEnvironment {
	/// Inspects a configuration key to get its value from all relevant scopes.
	async fn InspectConfigurationValue(
		&self,
		_Key:String,
		_Overrides:ConfigurationOverridesDTO,
	) -> Result<Option<InspectResultDataDTO>, CommonError> {
		warn!("[ConfigurationProvider] InspectConfigurationValue is not fully implemented.");
		Ok(None)
	}
}

/// An internal helper to read and parse a single JSON configuration file.
async fn ReadAndParseConfigurationFile(Environment:&MountainEnvironment, Path:&Option<PathBuf>) -> Value {
	if let Some(p) = Path {
		let FileSystemReader:Arc<dyn FileSystemReader> = Environment.Require();
		if let Ok(Bytes) = FileSystemReader.ReadFile(p).await {
			if let Ok(Value) = serde_json::from_slice(&Bytes) {
				return Value;
			} else {
				warn!("[ConfigurationProvider] Failed to parse JSON from config file: {}", p.display());
			}
		}
	}
	Value::Object(Map::new())
}

/// Logic to load and merge all configuration files into the effective
/// configuration stored in `ApplicationState`.
pub async fn InitializeAndMergeConfigurations(Environment:&MountainEnvironment) {
	info!("[ConfigurationProvider] Initializing and merging all configurations...");

	let UserSettingsPath = Environment
		.ApplicationHandle
		.path()
		.app_config_dir()
		.map(|p| p.join("settings.json"))
		.ok();

	let WorkSpaceSettingsPath = Environment.ApplicationState.WorkSpaceConfigurationPath.lock().unwrap().clone();

	let UserConfig = ReadAndParseConfigurationFile(Environment, &UserSettingsPath).await;
	let WorkSpaceConfig = ReadAndParseConfigurationFile(Environment, &WorkSpaceSettingsPath).await;

	let mut Merged = UserConfig.as_object().cloned().unwrap_or_default();
	if let Some(WorkSpaceMap) = WorkSpaceConfig.as_object() {
		for (k, v) in WorkSpaceMap {
			Merged.insert(k.clone(), v.clone());
		}
	}

	let FinalConfig = crate::ApplicationState::DTO::MergedConfigurationStateDTO::MergedConfigurationStateDTO::Create(
		Value::Object(Merged),
	);

	*Environment.ApplicationState.Configuration.lock().unwrap() = FinalConfig.clone();

	info!("[ConfigurationProvider] Configuration state updated and merged.");
}
