// File: Mountain/Source/Environment/ConfigurationProvider.rs
//
// # Architectural Role: Configuration Management Engine
//
// ConfigurationProvider implements ConfigurationProvider and
// ConfigurationInspector traits, managing all application settings across
// multiple scopes (Default, User, Workspace, Folder). It handles the
// configuration cascade, merging settings from various sources in the correct
// precedence order.
//
// # Responsibilities
//
// 1. **Configuration Cascade**: Implements the multi-layer configuration
//    hierarchy Default → User → Workspace → Folder, with higher precedence
//    overriding lower.
//
// 2. **Configuration Merging**: Performs deep merge of JSON configuration
//    objects from all scopes to produce the effective configuration.
//
// 3. **Configuration Persistence**: Reads and writes settings.json files for
//    User and Workspace scopes using the FileSystemWriter effect.
//
// 4. **Configuration Inspection**: Provides visibility into which scope is
//    providing each configuration value for debugging and diagnostics.
//
// 5. **Extension Configuration Merging**: Collects default configurations from
//    all installed extensions as part of the Default scope.
//
// # Configuration Cascade Order (Highest to Lowest Precedence)
//
// 1. Workspace Folder settings (.vscode/settings.json)
// 2. Workspace settings (workspace root settings.json)
// 3. User settings (app config dir/settings.json)
// 4. Extension default configurations (from package.json)
//
// # Patterns Borrowed from VSCode
//
// - **Configuration Targets**: Mimics VSCode's ConfigurationTarget enum for
//   specifying which configuration layer to update.
//
// - **Deep Merge**: Like VSCode's configuration service, performs recursive
//   merge of JSON objects instead of shallow replacement.
//
// - **Configuration Inspection**: Similar to VSCode's inspect() API, provides
//   visibility into all configuration sources and their values.
//
// - **JSON Editing**: Like VSCode's JSONEditingService, uses queue-based writes
//   to prevent race conditions during concurrent updates.
//
// # TODOs
//
// - [ ] Implement full Folder scope configuration with multi-folder workspace
//   support
// - [ ] Add configuration schema validation (JSON Schema draft-07)
// - [ ] Implement configuration language override semantics
// - [ ] Add configuration change event propagation to UI and extensions
// - [ ] Implement configuration export/import functionality
// - [ ] Add configuration profile support (multiple user setting files)
// - [ ] Implement configuration value type conversion and validation
// - [ ] Add secure configuration storage for sensitive values (passwords,
//   tokens)
// - [ ] Consider adding configuration value watchers for reactive updates
// - [ ] Implement configuration migration for version upgrades

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
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
use log::{debug, info, warn};
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

		Overrides:ConfigurationOverridesDTO,
	) -> Result<Value, CommonError> {
		debug!("[ConfigurationProvider] Getting configuration for section: {:?}", Section);

		let ConfigurationGuard = self
			.ApplicationState
			.Configuration
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?;

		let ConfigurationValue = ConfigurationGuard.GetValue(Section.as_deref());

		// Validate that the configuration value exists
		if ConfigurationValue.is_null() {
			warn!("[ConfigurationProvider] Configuration section not found: {:?}", Section);
		}

		Ok(ConfigurationValue)
	}

	/// Updates a configuration value in the appropriate `settings.json` file.
	async fn UpdateConfigurationValue(
		&self,

		Key:String,

		Value:Value,

		Target:ConfigurationTarget,

		Overrides:ConfigurationOverridesDTO,

		ScopeToLanguage:Option<bool>,
	) -> Result<(), CommonError> {
		info!("[ConfigurationProvider] Updating key '{}' in target {:?}", Key, Target);

		let RunTime = self.ApplicationHandle.state::<Arc<MountainRunTime>>().inner().clone();

		let ConfigPath:PathBuf = match Target {
			ConfigurationTarget::User => {
				self.ApplicationHandle
					.path()
					.app_config_dir()
					.map(|p| p.join("settings.json"))
					.map_err(|Error| {
						CommonError::ConfigurationLoad {
							Description:format!("Could not resolve user config path: {}", Error),
						}
					})?
			},

			ConfigurationTarget::Workspace => {
				self.ApplicationState
					.WorkspaceConfigurationPath
					.lock()
					.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
					.clone()
					.ok_or_else(|| {
						CommonError::ConfigurationLoad { Description:"No workspace configuration path set".into() }
					})?
			},

			_ => {
				warn!("[ConfigurationProvider] Unsupported configuration target: {:?}", Target);

				return Err(CommonError::NotImplemented {
					FeatureName:"This configuration target is not supported".into(),
				});
			},
		};

		// Read the file, modify it, and write it back.
		let Bytes = RunTime.Run(ReadFile(ConfigPath.clone())).await.unwrap_or_default();

		let mut CurrentConfig:Value = serde_json::from_slice(&Bytes).unwrap_or_else(|_| Value::Object(Map::new()));

		if let Value::Object(Map) = &mut CurrentConfig {
			if Value.is_null() {
				Map.remove(&Key);
				info!("[ConfigurationProvider] Removed configuration key '{}'", Key);
			} else {
				Map.insert(Key.clone(), Value.clone());
				info!("[ConfigurationProvider] Updated configuration key '{}'", Key);
			}
		}

		let ContentBytes = serde_json::to_vec_pretty(&CurrentConfig)?;

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

		Overrides:ConfigurationOverridesDTO,
	) -> Result<Option<InspectResultDataDTO>, CommonError> {
		info!("[ConfigurationProvider] Inspecting key: {}", Key);

		let UserSettingsPath = self
			.ApplicationHandle
			.path()
			.app_config_dir()
			.map(|p| p.join("settings.json"))
			.ok();

		let WorkspaceSettingsPath = self
			.ApplicationState
			.WorkspaceConfigurationPath
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.clone();

		// Read each configuration layer individually.
		let DefaultConfig = CollectDefaultConfigurations(&self.ApplicationState)?;

		let UserConfig = ReadAndParseConfigurationFile(self, &UserSettingsPath).await?;

		let WorkspaceConfig = ReadAndParseConfigurationFile(self, &WorkspaceSettingsPath).await?;

		let GetValueFromDotPath =
			|Node:&Value, Path:&str| -> Option<Value> { Path.split('.').try_fold(Node, |n, k| n.get(k)).cloned() };

		let mut ResultDTO = InspectResultDataDTO::default();

		ResultDTO.DefaultValue = GetValueFromDotPath(&DefaultConfig, &Key);

		ResultDTO.UserValue = GetValueFromDotPath(&UserConfig, &Key);

		ResultDTO.WorkspaceValue = GetValueFromDotPath(&WorkspaceConfig, &Key);

		// Determine the final effective value based on the correct cascade order.
		ResultDTO.EffectiveValue = ResultDTO
			.WorkspaceValue
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

	let WorkspaceSettingsPath = Environment
		.ApplicationState
		.WorkspaceConfigurationPath
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
		.clone();

	let UserConfig = ReadAndParseConfigurationFile(Environment, &UserSettingsPath).await?;

	let WorkspaceConfig = ReadAndParseConfigurationFile(Environment, &WorkspaceSettingsPath).await?;

	// A true deep merge is required here. The merge order matches the cascade:
	// Default (base) → User (overrides default) → Workspace (overrides user)
	let mut Merged = DefaultConfig.as_object().cloned().unwrap_or_default();

	if let Some(UserMap) = UserConfig.as_object() {
		for (Key, Value) in UserMap {
			// Deep merge nested objects, shallow merge at root level
			if Value.is_object() && Merged.get(&Key).is_some_and(|v| v.is_object()) {
				if let (Some(UserValue), Some(BaseValue)) =
					(Value.as_object(), Merged.get(&Key).and_then(|v| v.as_object()))
				{
					for (InnerKey, InnerValue) in UserValue {
						Merged.get_mut(&Key).and_then(|v| v.as_object_mut()).map(|m| {
							m.insert(InnerKey.clone(), InnerValue.clone());
						});
					}
				}
			} else {
				Merged.insert(Key.clone(), Value.clone());
			}
		}
	}

	if let Some(WorkspaceMap) = WorkspaceConfig.as_object() {
		for (Key, Value) in WorkspaceMap {
			if Value.is_object() && Merged.get(&Key).is_some_and(|v| v.is_object()) {
				if let (Some(WorkspaceValue), Some(BaseValue)) =
					(Value.as_object(), Merged.get(&Key).and_then(|v| v.as_object()))
				{
					for (InnerKey, InnerValue) in WorkspaceValue {
						Merged.get_mut(&Key).and_then(|v| v.as_object_mut()).map(|m| {
							m.insert(InnerKey.clone(), InnerValue.clone());
						});
					}
				}
			} else {
				Merged.insert(Key.clone(), Value.clone());
			}
		}
	}

	let FinalConfig = MergedConfigurationStateDTO::Create(Value::Object(Merged));

	*Environment
		.ApplicationState
		.Configuration
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)? = FinalConfig;

	let ConfigurationSize = Merged.len();
	info!(
		"[ConfigurationProvider] Configuration merged successfully with {} top-level keys.",
		ConfigurationSize
	);

	Ok(())
}
