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

use crate::{Environment::Utility, RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Updates a configuration value in the appropriate `settings.json` file.
pub(super) async fn update_configuration_value(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,
	key:String,
	value:Value,
	target:ConfigurationTarget,
	_overrides:ConfigurationOverridesDTO,
	_scope_to_language:Option<bool>,
) -> Result<(), CommonError> {
	dev_log!(
		"config",
		"[ConfigurationProvider] Updating key '{}' in target {:?}",
		key,
		target
	);

	let runtime = environment.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

	let config_path:PathBuf = match target {
		// Land treats `UserLocal` and `User` as the same `settings.json`
		// at the app-config dir. Stock VS Code differentiates them when
		// settings sync is on (UserLocal stays per-machine, User syncs);
		// Land has no sync backend, so the distinction is moot.
		ConfigurationTarget::UserLocal | ConfigurationTarget::User => {
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
				.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
				.clone()
				.ok_or_else(|| {
					CommonError::ConfigurationLoad { Description:"No workspace configuration path set".into() }
				})?
		},

		// `WorkspaceFolder` (multi-root) - write to
		// `<folder>/.vscode/settings.json` of the first workspace
		// folder. Multi-root extensions should pass the folder URI
		// in `_overrides.resource`; until that's plumbed through the
		// trait the first folder is the closest stable approximation.
		ConfigurationTarget::WorkspaceFolder => {
			let FoldersGuard = environment
				.ApplicationState
				.Workspace
				.WorkspaceFolders
				.lock()
				.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?;
			let First = FoldersGuard.first().ok_or_else(|| {
				CommonError::ConfigurationLoad {
					Description:"No workspace folders open for WorkspaceFolder target".into(),
				}
			})?;
			let FolderPath = First.URI.to_file_path().map_err(|_| {
				CommonError::ConfigurationLoad {
					Description:format!("Workspace folder URI is not a local path: {}", First.URI),
				}
			})?;
			FolderPath.join(".vscode").join("settings.json")
		},

		// `Memory` target only updates the in-memory configuration
		// state for the lifetime of the session - no disk write.
		// `SetGlobalValue` writes into the merged-config DTO; the
		// DTO is the same map `GetValue` reads from, so subsequent
		// `Inspect` / `Get` calls reflect the override immediately.
		ConfigurationTarget::Memory => {
			environment.ApplicationState.Configuration.SetGlobalValue(&key, value.clone());
			dev_log!(
				"config",
				"[ConfigurationProvider] Memory target: stored in-memory value for '{}'",
				key
			);
			return Ok(());
		},

		// `Default` and `Policy` are read-only by spec.
		ConfigurationTarget::Default | ConfigurationTarget::Policy => {
			return Err(CommonError::InvalidArgument {
				ArgumentName:"target".into(),
				Reason:format!("Configuration target {:?} is read-only", target),
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

	// Invalidate the parsed-settings.json cache so the very next
	// Inspect / merge re-reads from disk. Without this, the cached
	// parse from before this update could stick around for up to
	// 250 ms and feed stale values to the workbench until expiry.
	crate::Environment::ConfigurationProvider::Loading::ClearSettingsFileCache();

	// Re-merge all configurations to update the live state.
	crate::Environment::ConfigurationProvider::Loading::initialize_and_merge_configurations(environment).await?;

	Ok(())
}
