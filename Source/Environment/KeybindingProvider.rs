//! # KeybindingProvider (Environment)
//!
//! Implements the `KeybindingProvider` trait for `MountainEnvironment`,
//! providing keybinding resolution, conflict detection, and command activation
//! based on keyboard input.
//!
//! Keybindings are collected from three sources in ascending priority:
//! extension `contributes.keybindings`, dynamically registered entries
//! (`keybinding:add` / `RegisterExtensionKeybindings`, held in
//! `ApplicationState::Feature::Keybindings`), and user `keybindings.json`
//! overrides. Negative commands (prefixed with `-`) act as unbind rules
//! and remove the matching entry. Each emitted rule carries a `source`
//! field (`"extension:<id>"`, `"dynamic"`/`"dynamic:<id>"`, `"user"`) so
//! consumers can apply source-weighted precedence.
//!
//! ## When clause evaluation
//!
//! "When" clauses are boolean expressions over context keys that control
//! whether a keybinding is active. Examples:
//! - `"editorTextFocus && !inQuickOpen"` - only when editor has focus
//! - `"debugState != 'inactive'"` - only when debugging
//! - `"resourceLangId == python"` - only for Python files
//!
//! Parsing and evaluation live in `Environment::Utility::WhenClause`
//! (full expression grammar: `&&`/`||`/`!`, comparisons, `=~`, `in`).
//! Mountain has no live context-key store - Sky owns it - so resolution
//! against a context happens via the `keybinding:resolve` /
//! `keybinding:evaluateWhen` wire methods, which take a context snapshot
//! from the caller. `GetResolvedKeybinding` returns the merged rule set
//! with `when` preserved as source text for the renderer-side resolver.
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

// Still pending: localization, custom schemes (vim/emacs/sublime),
// keybinding recording, per-profile keybindings, export/import,
// search/discovery UI, telemetry.
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
struct KeybindingRule {
	key:String,

	command:String,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	when:Option<String>,

	#[serde(default, skip_serializing_if = "Option::is_none")]
	args:Option<Value>,

	/// Provenance tag added during resolution; absent in the raw
	/// contribution JSON, so deserialization defaults it.
	#[serde(default, skip_serializing_if = "Option::is_none")]
	source:Option<String>,
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
			.clone();

		for (Identifier, Extension) in Extensions.iter() {
			if let Some(Contributes) = Extension.Contributes.as_ref().and_then(|c| c.get("keybindings")) {
				if let Some(KeybindingsArray) = Contributes.as_array() {
					for KeybindingValue in KeybindingsArray {
						if let Ok(mut KeybindingRule) =
							serde_json::from_value::<KeybindingRule>(KeybindingValue.clone())
						{
							KeybindingRule.source = Some(format!("extension:{}", Identifier));

							let UniqueKey =
								format!("{}{}", KeybindingRule.key, KeybindingRule.when.as_deref().unwrap_or(""));

							ResolvedKeybindings.insert(UniqueKey, KeybindingRule);
						}
					}
				}
			}
		}

		// 2. Overlay dynamically registered entries (keybinding:add /
		// RegisterExtensionKeybindings). They outrank static extension
		// contributions but stay below the user's keybindings.json.
		for Entry in self.ApplicationState.Feature.Keybindings.GetAllKeybindings() {
			let Rule = KeybindingRule {
				key:Entry.Keybinding,
				command:Entry.CommandId,
				when:Entry.When,
				args:None,
				source:Some(
					Entry
						.Source
						.map(|Owner| format!("dynamic:{}", Owner))
						.unwrap_or_else(|| "dynamic".to_string()),
				),
			};

			let UniqueKey = format!("{}{}", Rule.key, Rule.when.as_deref().unwrap_or(""));

			ResolvedKeybindings.insert(UniqueKey, Rule);
		}

		// 3. Load and apply user-defined keybindings from keybindings.json
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
				for mut UserKeybinding in UserKeybindings {
					let UniqueKey = format!("{}{}", UserKeybinding.key, UserKeybinding.when.as_deref().unwrap_or(""));

					if UserKeybinding.command.starts_with('-') {
						// Unbind rule
						ResolvedKeybindings.remove(&UniqueKey);
					} else {
						// Override rule
						UserKeybinding.source = Some("user".to_string());

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

		let mut FinalRules:Vec<KeybindingRule> = ResolvedKeybindings.into_values().collect();

		// HashMap iteration order is nondeterministic; sort so repeated
		// calls (and conflict reports built on top) are stable.
		FinalRules.sort_by(|A, B| (&A.key, &A.command).cmp(&(&B.key, &B.command)));

		Ok(json!(FinalRules))
	}
}
