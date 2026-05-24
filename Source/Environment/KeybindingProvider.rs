//! # KeybindingProvider (Environment)
//!
//! Implements the `KeybindingProvider` trait for `MountainEnvironment`,
//! providing keybinding resolution, conflict detection, and command activation
//! based on keyboard input.
//!
//! Keybindings are collected from three sources in descending priority:
//! user `keybindings.json` overrides, extension `contributes.keybindings`,
//! and Mountain built-ins. Negative commands (prefixed with `-`) act as
//! unbind rules and remove the matching entry.
//!
//! ## When clause evaluation
//!
//! "When" clauses are boolean expressions over context keys that control
//! whether a keybinding is active. Examples:
//! - `"editorTextFocus && !inQuickOpen"` - only when editor has focus
//! - `"debugState != 'inactive'"` - only when debugging
//! - `"resourceLangId == python"` - only for Python files
//!
//! Current implementation stores when clauses but only partially evaluates
//! them. Full expression parsing and evaluation is pending.
//!
//! ## VS Code reference
//!
//! - `vs/platform/keybinding/common/keybinding.ts`
//! - `vs/platform/keybinding/common/keybindingResolver.ts`
//! - `vs/platform/keybinding/common/keybindingsRegistry.ts`
//! - `vs/platform/contextkey/common/contextkey.ts`

use std::{collections::HashMap, sync::Arc};

use CommonLibrary::{
	Effect::ApplicationRunTime::ApplicationRunTime as _,
	Error::CommonError::CommonError,
	FileSystem::ReadFile::ReadFile,
	Keybinding::KeybindingProvider::KeybindingProvider,
};
use async_trait::async_trait;
use serde_json::{Value, json};
use tauri::Manager;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

// TODO: full "when" clause expression parser/evaluator, precedence scoring
// algorithm, chord support ("Ctrl+K Ctrl+C"), platform modifier conversion
// (Cmd/Ctrl/Alt), conflict detection/warnings, localization, custom schemes
// (vim/emacs/sublime), keybinding recording, per-profile keybindings,
// export/import, search/discovery UI, telemetry.
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
		dev_log!("keybinding", "[KeybindingProvider] Resolving all keybindings...");

		let mut ResolvedKeybindings:HashMap<String, KeybindingRule> = HashMap::new();

		// 1. Collect default keybindings from extensions
		let Extensions = self
			.ApplicationState
			.Extension
			.ScannedExtensions
			.ScannedExtensions
			.lock()
			.map_err(Utility::ErrorMapping::MapApplicationStateLockErrorToCommonError)?
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
			.Path()
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
				dev_log!(
					"keybinding",
					"warn: [KeybindingProvider] Failed to parse user keybindings.json. It may be malformed."
				);
			}
		}

		let FinalRules:Vec<KeybindingRule> = ResolvedKeybindings.into_values().collect();

		Ok(json!(FinalRules))
	}
}
