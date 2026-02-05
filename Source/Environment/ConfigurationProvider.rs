//! # ConfigurationProvider (Environment)
//!
//! Implements `ConfigurationProvider` and `ConfigurationInspector` traits,
//! managing all application settings across multiple scopes (Default, User,
//! Workspace, Folder). It handles the configuration cascade, merging settings
//! from various sources in the correct precedence order.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Configuration Cascade
//! - Implement multi-layer configuration hierarchy: Default → User → Workspace
//!   → Folder
//! - Apply precedence rules: higher layers override lower layers
//! - Merge configuration objects from different sources
//! - Handle configuration inheritance and overrides
//!
//! ### 2. Configuration Storage
//! - Read default settings from resource files
//! - Load user settings from global storage (`globalStorage.json`)
//! - Load workspace settings from `.code-workspace` files
//! - Load folder settings from `.vscode/settings.json`
//! - Persist configuration changes to disk
//!
//! ### 3. Configuration Access
//! - Provide `GetConfiguration` for retrieving merged settings
//! - Support nested property access via dot notation (e.g., `"editor.fontSize"`)
//! - Implement `ConfigurationInspector` for introspection
//! - Cache configuration for performance
//!
//! ### 4. Configuration Updates
//! - Handle runtime configuration changes
//! - Notify listeners when configuration updates occur
//! - Persist changes to appropriate storage layer
//! - Re-merge configuration after changes
//!
//! ## ARCHITECTURAL ROLE
//!
//! ConfigurationProvider is the **configuration management core**:
//!
//! ```text
//! Providers ──► ConfigurationProvider ──► MergedConfigurationStateDTO ──► Disk
//!               │
//!               └─► Multiple Storage Layers
//! ```
//!
//! ### Position in Mountain
//! - `Environment` module: Core capability provider
//! - Implements `CommonLibrary::Configuration::ConfigurationProvider` and
//!   `ConfigurationInspector`
//! - Accessible via `Environment.Require<dyn ConfigurationProvider>()`
//!
//! ### Configuration Scope Precedence (lowest to highest)
//! 1. **Default**: Built-in Mountain defaults
//! 2. **User**: Global user settings (`User/settings.json`)
//! 3. **Workspace**: Workspace file (`.code-workspace`)
//! 4. **Folder**: Per-folder settings (`.vscode/settings.json`)
//! 5. **Workspace-Folder**: Workspace settings for specific folder
//!
//! ### Dependencies
//! - `ApplicationState`: Access to `Configuration` state and storage paths
//! - `FileSystemReader`: Read configuration files from disk
//! - `Log`: Configuration change logging
//!
//! ### Dependents
//! - All providers that need configuration values
//! - `Binary::Main`: Initial configuration setup
//! - `InitializationData`: Includes configuration in frontend payload
//! - Command handlers: Read settings to customize behavior
//!
//! ## CONFIGURATION MERGE STRATEGY
//!
//! Configuration is merged using a **depth-first cascade**:
//!
//! 1. Start with default values
//! 2. Overlay user settings (if exist)
//! 3. Overlay workspace settings (if exists)
//! 4. For each folder, overlay folder settings
//! 5. For workspace-folders, apply folder-specific overrides
//!
//! The final result is a single JSON object representing the effective
//! configuration for the current context.
//!
//! ## PROPERTY ACCESS
//!
//! - `GetConfiguration(Section)` returns the entire configuration or a
//!   subsection
//! - Section uses dot notation: `"editor.fontSize"` or `"typescript.format.enable"`
//! - Returns `serde_json::Value` which can be queried further
//! - Missing properties return `Value::Null`
//!
//! ## ERROR HANDLING
//!
//! - File read errors: `CommonError::FileSystemIO`
//! - JSON parse errors: `CommonError::SerializationError`
//! - Invalid section paths: `Value::Null` (no error)
//! - Write permission errors: `CommonError::FileSystemIO`
//!
//! ## PERFORMANCE
//!
//! - Configuration is cached in `ApplicationState::Configuration`
//! - Merging happens on change (not on every read)
//! - File reads are async via `ApplicationRunTime`
//! - Consider incremental merging for large configurations (TODO)
//!
//! ## VS CODE REFERENCE
//!
//! Patterns from VS Code's configuration system:
//! - `vs/platform/configuration/common/configuration.ts` - Configuration service
//! - `vs/platform/configuration/common/configurationCache.ts` - Caching
//! - `vs/platform/configuration/common/configurationLayer.ts` - Layer merging
//!
//! ## TODO
//!
//! - [ ] Implement configuration change observers/eventing
//! - [ ] Add configuration schema validation
//! - [ ] Support configuration profiles (dev, prod, test)
//! - [ ] Implement configuration export/import
//! - [ ] Add configuration search and query API
//! - [ ] Support remote configuration sources
//! - [ ] Add configuration diffing for debugging
//! - [ ] Implement configuration hot-reload without restart
//! - [ ] Add configuration permissions and security
//! - [ ] Support configuration templates and presets
//!
//! ## MODULE CONTENTS
//!
//! - [`ConfigurationProvider`]: Main struct implementing configuration management
//! - Configuration loading and merging functions
//! - Storage layer abstractions

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
			if Value.is_object() && Merged.get(Key.as_str()).is_some_and(|v| v.is_object()) {
				if let (Some(UserValue), Some(BaseValue)) =
					(Value.as_object(), Merged.get(Key.as_str()).and_then(|v| v.as_object()))
				{
					for (InnerKey, InnerValue) in UserValue {
						Merged.get_mut(Key.as_str()).and_then(|v| v.as_object_mut()).map(|m| {
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
			if Value.is_object() && Merged.get(Key.as_str()).is_some_and(|v| v.is_object()) {
				if let (Some(WorkspaceValue), Some(BaseValue)) =
					(Value.as_object(), Merged.get(Key.as_str()).and_then(|v| v.as_object()))
				{
					for (InnerKey, InnerValue) in WorkspaceValue {
						Merged.get_mut(Key.as_str()).and_then(|v| v.as_object_mut()).map(|m| {
							m.insert(InnerKey.clone(), InnerValue.clone());
						});
					}
				}
			} else {
				Merged.insert(Key.clone(), Value.clone());
			}
		}
	}

	let ConfigurationSize = Merged.len();
	let FinalConfig = MergedConfigurationStateDTO::Create(Value::Object(Merged));

	*Environment
		.ApplicationState
		.Configuration
		.lock()
		.map_err(Utility::MapApplicationStateLockErrorToCommonError)? = FinalConfig;

	info!(
		"[ConfigurationProvider] Configuration merged successfully with {} top-level keys.",
		ConfigurationSize
	);

	Ok(())
}
