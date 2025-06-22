// File: Mountain/Source/Environment/KeybindingProvider.rs
// Role: Implements the `KeybindingProvider` trait for the
// `MountainEnvironment`. Responsibilities:
//   - Resolve the final, effective keymap for the application.
//   - Collect default keybindings contributed by all scanned extensions.
//   - Read and apply user-defined keybindings from `keybindings.json`, handling
//     overrides and unbindings.

//! # KeybindingProvider Implementation
//!
//! Implements the `KeybindingProvider` trait for the `MountainEnvironment`.

use std::{collections::HashMap, sync::Arc};

use Common::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::ReadFile::ReadFile,
	Keybinding::KeybindingProvider::KeybindingProvider,
};
use async_trait::async_trait;
use log::info;
use serde_json::{Value, json};
use tauri::Manager;

use super::MountainEnvironment::MountainEnvironment;
use crate::RunTime::ApplicationRunTime::ApplicationRunTime;

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct KeybindingRule {
	key:String,

	command:String,

	when:Option<String>,

	args:Option<Value>,
}

#[async_trait]
impl KeybindingProvider for MountainEnvironment {
	async fn GetResolvedKeybinding(&self) -> Result<Value, CommonError> {
		info!("[KeybindingProvider] Resolving all keybindings...");

		let mut resolved_keybindings:HashMap<String, KeybindingRule> = HashMap::new();

		// 1. Collect default keybindings from extensions
		{
			let extensions = self.ApplicationState.ScannedExtensions.lock().unwrap();

			for ext in extensions.values() {
				if let Some(contributes) = ext.Contributes.as_ref().and_then(|c| c.get("keybindings")) {
					if let Some(keybindings_array) = contributes.as_array() {
						for kb_val in keybindings_array {
							if let Ok(kb_rule) = serde_json::from_value::<KeybindingRule>(kb_val.clone()) {
								// Use key+when as a unique identifier for the rule
								let unique_key = format!("{}{}", kb_rule.key, kb_rule.when.as_deref().unwrap_or(""));

								resolved_keybindings.insert(unique_key, kb_rule);
							}
						}
					}
				}
			}

			// extensions lock is dropped here
		}

		// 2. Load and apply user-defined keybindings from keybindings.json
		let user_keybindings_path = self
			.ApplicationHandle
			.path()
			.app_config_dir()
			.map_err(|e| CommonError::ConfigurationLoad { Description:format!("Cannot find app config dir: {}", e) })?
			.join("keybindings.json");

		let runtime = self.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

		if let Ok(content) = runtime.Run(ReadFile(user_keybindings_path)).await {
			if let Ok(user_keybindings) = serde_json::from_slice::<Vec<KeybindingRule>>(&content) {
				for user_kb in user_keybindings {
					let unique_key = format!("{}{}", user_kb.key, user_kb.when.as_deref().unwrap_or(""));

					if user_kb.command.starts_with('-') {
						// This is an "unbind" rule
						resolved_keybindings.remove(&unique_key);
					} else {
						// This rule overrides any existing default
						resolved_keybindings.insert(unique_key, user_kb);
					}
				}
			}
		}

		let final_rules:Vec<KeybindingRule> = resolved_keybindings.into_values().collect();

		Ok(json!(final_rules))
	}
}
