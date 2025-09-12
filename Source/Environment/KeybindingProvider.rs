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

#![allow(non_snake_case, non_camel_case_types)]

use std::{collections::HashMap, sync::Arc};

use Common::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::ReadFile::ReadFile,
	Keybinding::KeybindingProvider::KeybindingProvider,
};
use async_trait::async_trait;
use log::{info, warn};
use serde_json::{Value, json};
use tauri::Manager;

use super::{MountainEnvironment::MountainEnvironment, Utility};
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

		let mut ResolvedKeybindings:HashMap<String, KeybindingRule> = HashMap::new();

		// 1. Collect default keybindings from extensions
		let Extensions = self
			.ApplicationState
			.ScannedExtensions
			.lock()
			.map_err(Utility::MapApplicationStateLockErrorToCommonError)?
			.clone();

		for Extension in Extensions.values() {
			if let Some(Contributes) = Extension.Contributes.as_ref().and_then(|c| c.get("keybindings")) {
				if let Some(KeybindingsArray) = Contributes.as_array() {
					for KeybindingValue in KeybindingsArray {
						if let Ok(KeybindingRule) = serde_json::from_value::<KeybindingRule>(KeybindingValue.clone()) {
							let UniqueKey =
								format!("{}{}", KeybindingRule.key, KeybindingRule.when.as_deref().unwrap_or(""));

							ResolvedKeybindings.insert(UniqueKey, KeybindingRule);
						}
					}
				}
			}
		}

		// 2. Load and apply user-defined keybindings from keybindings.json
		let UserKeybindingsPath = self
			.ApplicationHandle
			.path()
			.app_config_dir()
			.map_err(|Error| {
				CommonError::ConfigurationLoad { Description:format!("Cannot find app config dir: {}", Error) }
			})?
			.join("keybindings.json");

		let RunTime = self.ApplicationHandle.state::<Arc<ApplicationRunTime>>().inner().clone();

		if let Ok(Content) = RunTime.Run(ReadFile(UserKeybindingsPath)).await {
			if let Ok(UserKeybindings) = serde_json::from_slice::<Vec<KeybindingRule>>(&Content) {
				for UserKeybinding in UserKeybindings {
					let UniqueKey = format!("{}{}", UserKeybinding.key, UserKeybinding.when.as_deref().unwrap_or(""));

					if UserKeybinding.command.starts_with('-') {
						// Unbind rule
						ResolvedKeybindings.remove(&UniqueKey);
					} else {
						// Override rule
						ResolvedKeybindings.insert(UniqueKey, UserKeybinding);
					}
				}
			} else {
				warn!("[KeybindingProvider] Failed to parse user keybindings.json. It may be malformed.");
			}
		}

		let FinalRules:Vec<KeybindingRule> = ResolvedKeybindings.into_values().collect();

		Ok(json!(FinalRules))
	}
}
