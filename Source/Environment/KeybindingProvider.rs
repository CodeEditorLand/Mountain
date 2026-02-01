// File: Mountain/Source/Environment/KeybindingProvider.rs
// Role: Implements the `KeybindingProvider` trait for the
// `MountainEnvironment`. Responsibilities:
//   - Resolve the final, effective keymap for the application.
//   - Collect default keybindings contributed by all scanned extensions.
//   - Read and apply user-defined keybindings from `keybindings.json`, handling
//     overrides and unbindings.
//   - Parse and resolve keybinding combinations (modifiers + keys).
//   - Evaluate "when" clauses to determine keybinding activation context.
//   - Handle keybinding conflicts with priority-based resolution.
//   - Support platform-specific keybindings.
//
// TODOs:
//   - Implement complete when clause evaluation (context key expressions)
//   - Add keybinding scoring for conflict resolution
//   - Support keybinding localization
//   - Implement platform-specific modifier key conversion (Cmd/Ctrl)
//   - Add keybinding conflict detection and warnings
//   - Implement keybinding chords (multi-key sequences)
//   - Support custom keybinding schemes
//   - Add keybinding validation and syntax error reporting
//   - Implement keybinding telemetry for usage tracking
//   - Support keybinding migration across versions
//
// Inspired by VSCode's keybinding service which:
// - Evaluates complex when clause expressions
// - Resolves conflicts with scoring (default > extension > user overrides)
// - Supports platform-specific transformations
// - Handles keybinding chords
// - Provides keybinding resolution diagnostics

//! # KeybindingProvider Implementation
//!
//! Implements the `KeybindingProvider` trait for the `MountainEnvironment`.
//!
//! ## Keybinding Resolution Strategy
//!
//! 1. Collect default keybindings from all enabled extensions
//! 2. Apply user-defined keybindings from `keybindings.json`
//! 3. Resolve conflicts using priority rules:
//!    - User keybindings override system defaults
//!    - Negative commands (starting with `-`) unbind keys
//!    - Higher priority values win in cases of ambiguity
//! 4. Evaluate when clauses at runtime to filter active keybindings
//!
//! ## When Clause Evaluation
//!
//! When clauses are boolean expressions controlling when a keybinding
//! is active. Examples:
//! - `"editorTextFocus && !inQuickOpen"` - Only when editor has focus
//! - `"debugState != 'inactive'"` - Only when debugging
//!
//! Current implementation stores when clauses but only partially
//! evaluates them. Full expression evaluation is pending.

use std::{collections::HashMap, sync::Arc};

use CommonLibrary::{
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
