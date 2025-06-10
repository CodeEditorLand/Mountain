use std::{path::PathBuf, sync::Arc};

use Common::{
	config::dto::{ConfigurationOverridesDto, ConfigurationTarget, InspectResultDataDto},
	error::CommonError,
	fs::{FileSystemReader, FileSystemWriter},
};
use log::{debug, error, info};
use serde_json::{Map, Value, json};
use tauri::{ApplicationHandle, Manager, RunTime};

// @module ConfigLogic
// @description Contains the core logic for configuration management, including
// reading, merging, updating, and inspecting settings from various sources.
use crate::{
	ApplicationState::{ApplicationState::ApplicationState, DTO::MergedConfigurationStateDto},
	Handler::error_utils,
	environment::Utils,
	vine::{self, client},
};

async fn read_and_parse_config(fs_reader:&Arc<dyn FileSystemReader>, path:&Option<PathBuf>) -> Value {
	if let Some(p) = path {
		if let Ok(bytes) = fs_reader.ReadFile(p).await {
			if let Ok(val) = serde_json::from_slice(&bytes) {
				return val;
			}
		}
	}
	Value::Object(Map::new())
}

// Logic to load and merge all configuration files into the effective
// configuration stored in ApplicationState. This is called at startup and after
// any settings file changes.
pub async fn InitializeConfiguration<R:RunTime>(ApplicationHandle:&ApplicationHandle<R>, AppStateInstance:&ApplicationState) {
	info!("[ConfigLogic] Initializing and merging all configurations...");
	let environment:tauri::State<'_, Arc<crate::environment::MountainEnvironment::MountainEnvironment>> =
		ApplicationHandle.state();
	let fs_reader:Arc<dyn FileSystemReader> = environment.Require();

	let user_settings_path = ApplicationHandle.path_resolver().app_config_dir().map(|p| p.join("settings.json"));
	let workspace_settings_path = AppStateInstance.WorkspaceConfigurationPath.lock().unwrap().clone();

	let user_config = read_and_parse_config(&fs_reader, &user_settings_path).await;
	let workspace_config = read_and_parse_config(&fs_reader, &workspace_settings_path).await;

	// A real implementation would also load default and folder-level settings.
	// The merge order is critical: workspace settings override user settings.
	let mut merged = user_config.as_object().cloned().unwrap_or_default();
	if let Some(workspace_map) = workspace_config.as_object() {
		for (k, v) in workspace_map {
			merged.insert(k.clone(), v.clone());
		}
	}

	let final_config = MergedConfigurationStateDto::New(Value::Object(merged));
	*AppStateInstance.Configuration.lock().unwrap() = final_config.clone();

	// Notify Cocoon of the change
	let payload = json!({ "keys": [], "source": 0 }); // Simplified
	client::SendNotification(
		"cocoon-main",
		"$acceptConfigurationChanged".to_string(),
		json!([payload, final_config]),
	)
	.await
	.ok();
}

// Logic to retrieve a configuration value from the cached, merged
// configuration.
pub async fn GetConfigurationValueLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Section:Option<String>,
	_Overrides:ConfigurationOverridesDto,
) -> Result<Value, CommonError> {
	debug!("[ConfigLogic] Getting configuration for section: {:?}", Section);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	let ConfigGuard = AppStateInstance
		.Configuration
		.lock()
		.map_err(Utils::MapAppStateLockErrorToCommonError)?;
	Ok(ConfigGuard.GetValue(Section.as_deref()))
}

// Logic to update a configuration value in the appropriate settings.json file.
pub async fn UpdateConfigurationValueLogic<R:RunTime>(
	ApplicationHandle:&ApplicationHandle<R>,
	Key:String,
	ValueToSet:Value,
	Target:ConfigurationTarget,
	_Overrides:ConfigurationOverridesDto,
	_ScopeToLanguage:Option<bool>,
) -> Result<(), CommonError> {
	info!("[ConfigLogic] Updating configuration key '{}' in target {:?}", Key, Target);
	let AppStateInstance = ApplicationHandle.state::<ApplicationState>();
	let environment:tauri::State<'_, Arc<crate::environment::MountainEnvironment::MountainEnvironment>> =
		ApplicationHandle.state();
	let fs_reader:Arc<dyn FileSystemReader> = environment.Require();
	let fs_writer:Arc<dyn FileSystemWriter> = environment.Require();

	let config_path = match Target {
		ConfigurationTarget::User => ApplicationHandle.path_resolver().app_config_dir().map(|p| p.join("settings.json")),
		ConfigurationTarget::Workspace => AppStateInstance.WorkspaceConfigurationPath.lock().unwrap().clone(),
		_ => return Err(CommonError::NotImplemented { FeatureName:"Configuration target not supported".into() }),
	};

	if let Some(path) = config_path {
		let mut current_config = read_and_parse_config(&fs_reader, &Some(path.clone())).await;
		if let Some(map) = current_config.as_object_mut() {
			// A more robust implementation would handle nested keys.
			map.insert(Key, ValueToSet);
		}
		let content_bytes = serde_json::to_vec_pretty(&current_config)?;
		fs_writer.WriteFile(&path, content_bytes, true, true).await?;
		// After writing, trigger a full reload to update the in-memory state.
		InitializeConfiguration(ApplicationHandle, &AppStateInstance).await;
	}
	Ok(())
}

// Logic to inspect a configuration value from all sources.
pub async fn InspectConfigurationValueLogic<R:RunTime>(
	_ApplicationHandle:&ApplicationHandle<R>,
	_Key:String,
	_Overrides:ConfigurationOverridesDto,
) -> Result<Option<InspectResultDataDto>, CommonError> {
	// This is a complex operation requiring reading from all config files
	// without merging them to build the final DTO.
	Ok(None)
}
