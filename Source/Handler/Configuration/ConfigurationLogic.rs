// @module ConfigurationLogic
// @description Contains the core logic for configuration management, including
// reading, merging, updating, and inspecting settings from various sources.

use std::{path::PathBuf, sync::Arc};

use Common::{
	config::DTO::{ConfigurationOverridesDTO, ConfigurationTarget, InspectResultDataDTO},
	error::CommonError,
	fs::{FileSystemReader, FileSystemWriter},
};
use log::{debug, error, info, warn};
use serde_json::{Map, Value};
use tauri::{AppHandle, Manager, Runtime};

use crate::{
	ApplicationState::{ApplicationState::ApplicationState, DTO::MergedConfigurationStateDTO},
	Environment::MountainEnvironment,
	Vine::client,
};

// Internal helper to read and parse a single JSON configuration file.
async fn read_and_parse_config(fs_reader:&Arc<dyn FileSystemReader>, path:&Option<PathBuf>) -> Value {
	if let Some(p) = path {
		if let Ok(bytes) = fs_reader.ReadFile(p).await {
			if let Ok(val) = serde_json::from_slice(&bytes) {
				return val;
			} else {
				warn!("[ConfigurationLogic] Failed to parse JSON from config file: {}", p.display());
			}
		}
	}
	Value::Object(Map::new())
}

// Logic to load and merge all configuration files into the effective
// configuration stored in `ApplicationState`. This is called at startup and
// after any settings file changes.
pub async fn InitializeConfiguration<R:Runtime>(app_handle:&AppHandle<R>, app_state:&ApplicationState) {
	info!("[ConfigurationLogic] Initializing and merging all configurations...");
	let Environment:tauri::State<'_, Arc<MountainEnvironment>> = app_handle.state();
	let fs_reader:Arc<dyn FileSystemReader> = Environment.Require();

	let user_settings_path = app_handle.path_resolver().app_config_dir().map(|p| p.join("settings.json"));
	let workspace_settings_path = app_state.WorkspaceConfigurationPath.lock().unwrap().clone();

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

	let final_config = MergedConfigurationStateDTO::New(Value::Object(merged));
	*app_state.Configuration.lock().unwrap() = final_config.clone();

	// Notify Cocoon of the change
	let payload = serde_json::json!({ "keys": [], "source": 0 }); // Simplified
	client::SendNotification(
		"cocoon-main".to_string(),
		"$acceptConfigurationChanged".to_string(),
		serde_json::json!([payload, final_config]),
	)
	.await
	.unwrap_or_else(|e| warn!("[ConfigurationLogic] Failed to notify Cocoon of config change: {}", e));
}

// Logic to retrieve a configuration value from the cached, merged
// configuration.
pub async fn GetConfigurationValueLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	section:Option<String>,
	_overrides:ConfigurationOverridesDTO,
) -> Result<Value, CommonError> {
	debug!("[ConfigurationLogic] Getting configuration for section: {:?}", section);
	let app_state = app_handle.state::<ApplicationState>();
	let config_guard = app_state
		.Configuration
		.lock()
		.map_err(crate::Environment::Utility::MapAppStateLockErrorToCommonError)?;
	Ok(config_guard.GetValue(section.as_deref()))
}

// Logic to update a configuration value in the appropriate `settings.json`
// file.
pub async fn UpdateConfigurationValueLogic<R:Runtime>(
	app_handle:&AppHandle<R>,
	key:String,
	value_to_set:Value,
	target:ConfigurationTarget,
	_overrides:ConfigurationOverridesDTO,
	_scope_to_language:Option<bool>,
) -> Result<(), CommonError> {
	info!("[ConfigurationLogic] Updating configuration key '{}' in target {:?}", key, target);
	let app_state = app_handle.state::<ApplicationState>();
	let Environment:tauri::State<'_, Arc<MountainEnvironment>> = app_handle.state();
	let fs_reader:Arc<dyn FileSystemReader> = Environment.Require();
	let fs_writer:Arc<dyn FileSystemWriter> = Environment.Require();

	let config_path = match target {
		ConfigurationTarget::User => app_handle.path_resolver().app_config_dir().map(|p| p.join("settings.json")),
		ConfigurationTarget::Workspace => app_state.WorkspaceConfigurationPath.lock().unwrap().clone(),
		_ => return Err(CommonError::NotImplemented { FeatureName:"Configuration target not supported".into() }),
	};

	if let Some(path) = config_path {
		let mut current_config = read_and_parse_config(&fs_reader, &Some(path.clone())).await;

		if let Value::Object(map) = &mut current_config {
			// A more robust implementation would handle nested keys like "a.b.c".
			if value_to_set.is_null() {
				map.remove(&key);
			} else {
				map.insert(key, value_to_set);
			}
		}

		let content_bytes = serde_json::to_vec_pretty(&current_config)
			.map_err(|e| CommonError::SerdeError { Description:e.to_string() })?;
		fs_writer.WriteFile(&path, content_bytes, true, true).await?;

		// After writing, trigger a full reload to update the in-memory state and notify
		// sidecars.
		InitializeConfiguration(app_handle, &app_state).await;
		Ok(())
	} else {
		Err(CommonError::ConfigUpdate {
			Key:key,
			Description:format!("Configuration target {:?} is not available.", target),
		})
	}
}

// Logic to inspect a configuration value from all sources.
pub async fn InspectConfigurationValueLogic<R:Runtime>(
	_app_handle:&AppHandle<R>,
	_key:String,
	_overrides:ConfigurationOverridesDTO,
) -> Result<Option<InspectResultDataDTO>, CommonError> {
	// This is a complex operation requiring reading from all config files
	// without merging them to build the final DTO. It is a stub for now.
	warn!("[ConfigurationLogic] `InspectConfigurationValueLogic` is not fully implemented.");
	Ok(None)
}
