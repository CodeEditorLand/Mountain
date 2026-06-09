//! Configuration value update and persistence.
//!
//! Implements `UpdateConfigurationValue` for `MountainEnvironment`.
//! Resolves the write target to a concrete `settings.json` path,
//! performs a read-modify-write, then invalidates the parse cache and
//! triggers a full re-merge so subsequent reads reflect the change.
//!
//! ## Target resolution
//!
//! | `ConfigurationTarget` | Write destination |
//! |---|---|
//! | `User` / `UserLocal` | `<app-config>/settings.json` |
//! | `Workspace` | workspace `settings.json` |
//! | `WorkspaceFolder` | `<first-folder>/.vscode/settings.json` |
//! | `Memory` | in-memory merged map only; no disk write |
//! | `Default` / `Policy` | error - read-only by spec |
//!
//! Passing `Value::Null` as the new value removes the key from the
//! target file rather than writing a `null` literal, matching
//! VS Code's "reset to default" behaviour.

use std::{path::PathBuf, sync::Arc};

use CommonLibrary::{
	Configuration::DTO::{
		ConfigurationOverridesDTO::ConfigurationOverridesDTO,
		ConfigurationTarget::ConfigurationTarget,
	},
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::{ReadFile::ReadFile, WriteFileBytes::WriteFileBytes},
	IPC::SkyEvent::SkyEvent,
};
use serde_json::{Map, Value};
use tauri::Manager;

use crate::{Environment::Utility, IPC::SkyEmit::LogSkyEmit, RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

/// Updates a configuration value in the appropriate `settings.json` file.
pub(super) async fn update_configuration_value(
	environment:&crate::Environment::MountainEnvironment::MountainEnvironment,

	key:String,

	value:Value,

	target:ConfigurationTarget,

	overrides:ConfigurationOverridesDTO,

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
			let FoldersGuard = environment.ApplicationState.Workspace.WorkspaceFolders.lock();

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

	if let Value::Object(ref mut RootMap) = current_config {
		if let Some(LangId) = overrides.OverrideIdentifier.as_deref().filter(|S| !S.is_empty()) {
			// Language-scoped override: write into `"[<langId>]": { key: value }`.
			// This is how VS Code stores per-language defaults:
			//   `prettier-vscode` sets `"[typescript]": { "editor.defaultFormatter": "..."
			// }`   `vscode-eslint` sets `"[javascript]": { "editor.codeActionsOnSave":
			// {...} }`
			let ScopeKey = format!("[{}]", LangId);

			let LangScope = RootMap.entry(ScopeKey.clone()).or_insert_with(|| Value::Object(Map::new()));

			if let Value::Object(LangMap) = LangScope {
				if value.is_null() {
					LangMap.remove(&key);

					if LangMap.is_empty() {
						RootMap.remove(&ScopeKey);
					}

					dev_log!("config", "[ConfigurationProvider] Removed '[{}]' key '{}'", LangId, key);
				} else {
					LangMap.insert(key.clone(), value.clone());

					dev_log!("config", "[ConfigurationProvider] Updated '[{}]' key '{}'", LangId, key);
				}
			}
		} else {
			// Top-level key - standard behaviour.
			if value.is_null() {
				RootMap.remove(&key);

				dev_log!("config", "[ConfigurationProvider] Removed configuration key '{}'", key);
			} else {
				RootMap.insert(key.clone(), value.clone());

				dev_log!("config", "[ConfigurationProvider] Updated configuration key '{}'", key);
			}
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
	crate::Environment::ConfigurationProvider::Loading::Fn(environment).await?;

	// Notify Sky (the VS Code workbench) so its ConfigurationService rebuilds
	// its cached model. Without this the Settings UI and workspace inspectors
	// don't reflect extension-triggered writes until a window reload.
	let EmitPayload = serde_json::json!({ "keys": [key] });

	if let Err(Error) = LogSkyEmit(
		&environment.ApplicationHandle,
		SkyEvent::ConfigurationChanged.AsStr(),
		EmitPayload,
	) {
		dev_log!(
			"config",
			"warn: [ConfigurationProvider] sky://configuration/changed emit failed: {}",
			Error
		);
	}

	// Also notify Cocoon's extension host so its ConfigCache invalidates the
	// affected key and re-primes. Without this, extensions calling
	// `workspace.getConfiguration().get(key)` after an update continue reading
	// the stale cached value until the next full re-merge notification.
	// Using fire-and-forget: the write is already durable; a Vine failure here
	// must not roll back the successful disk write.
	let NotifyKey = key.clone();

	tokio::spawn(async move {
		if let Err(Error) = ::Vine::Client::SendNotification::Fn(
			"cocoon-main".to_string(),
			"configuration.change".to_string(),
			serde_json::json!({ "keys": [NotifyKey] }),
		)
		.await
		{
			crate::dev_log!(
				"config",
				"warn: [ConfigurationProvider] configuration.change Cocoon notify failed: {:?}",
				Error
			);
		}
	});

	Ok(())
}
