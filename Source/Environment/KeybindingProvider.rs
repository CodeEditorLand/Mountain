//! # KeybindingProvider (Environment)
//!
//! Implements the `KeybindingProvider` trait for `MountainEnvironment`,
//! providing keybinding resolution, conflict detection, and command activation
//! based on keyboard input.
//!
//! ## RESPONSIBILITIES
//!
//! ### 1. Keybinding Collection
//! - Gather default keybindings from all extensions (`contributes.keybindings`)
//! - Load user-defined keybindings from `keybindings.json`
//! - Include built-in Mountain keybindings
//! - Support platform-specific keybinding variants
//!
//! ### 2. Keybinding Resolution
//! - Parse keybinding combinations (modifiers + keys)
//! - Evaluate "when" clauses for context activation
//! - Resolve conflicts using priority rules:
//!   - User keybindings override extension keybindings
//!   - Extension keybindings override defaults
//!   - Within same priority, use specificity scoring
//!
//! ### 3. Command Activation
//! - Match incoming keyboard events to keybindings
//! - Execute bound commands with proper arguments
//! - Handle keybinding chords (multi-key sequences)
//! - Support keybinding cancellation (Esc, etc.)
//!
//! ### 4. Platform Adaptation
//! - Convert between platform-specific modifiers (Cmd on macOS, Ctrl on
//!   Windows/Linux)
//! - Support international keyboard layouts
//! - Handle platform-exclusive keybindings
//!
//! ## ARCHITECTURAL ROLE
//!
//! KeybindingProvider is the **keyboard input mapper**:
//!
//! ```text
//! Keyboard Event ──► KeybindingProvider ──► Resolved Keybinding ──► CommandExecutor
//! ```
//!
//! ### Position in Mountain
//! - `Environment` module: Input handling capability
//! - Implements `CommonLibrary::Keybinding::KeybindingProvider` trait
//! - Accessible via `Environment.Require<dyn KeybindingProvider>()`
//!
//! ### Keybinding Source Precedence (highest to lowest)
//! 1. **User keybindings**: `keybindings.json` with overrides and unbindings
//! 2. **Extension keybindings**: From `contributes.keybindings` in package.json
//! 3. **Built-in keybindings**: Mountain default shortcuts
//!
//! ### Conflict Resolution
//!
//! When multiple keybindings match an input:
//! 1. **Priority**: User > Extension > Built-in
//! 2. **Specificity**: More specific "when" clause wins
//! 3. **Scoring**: (TODO) Calculate score based on:
//!    - Number of modifiers (more is more specific)
//!    - Presence of "when" clause conditions
//!    - Platform specificity
//!
//! ## DATA MODEL
//!
//! A `Keybinding` consists of:
//! - `Command`: Command ID to execute
//! - `Keybinding`: Keyboard shortcut string (e.g., "Ctrl+Shift+P")
//! - `When`: Context expression (e.g., "editorTextFocus")
//! - `Source`: Where it came from (user, extension, built-in)
//! - `Weight`: Priority weight for conflict resolution
//!
//! ## WHEN CLAUSE EVALUATION
//!
//! "When" clauses are boolean expressions over context keys:
//! - `editorTextFocus`: Editor has focus
//! - `editorHasSelection`: Text is selected
//! - `resourceLangId == python`: File is Python
//! - `!editorHasSelection`: Negation
//! - `editorTextFocus && !editorHasSelection`: AND combination
//!
//! Context keys are set by providers based on UI state:
//! - Focus state, language, file type, editor mode, etc.
//!
//! ## PERFORMANCE
//!
//! - Keybinding lookup should be fast (HashMap or trie-based)
//! - "When" clause evaluation should be efficient (pre-parsed expressions)
//! - Consider caching resolved keybindings per context
//! - Platform-specific lookup avoids runtime conversion overhead
//!
//! ## VS CODE REFERENCE
//!
//! Borrowed from VS Code's keybinding system:
//! - `vs/platform/keybinding/common/keybinding.ts` - Keybinding data model
//! - `vs/platform/keybinding/common/keybindingResolver.ts` - Resolution logic
//! - `vs/platform/keybinding/common/keybindingsRegistry.ts` - Registry
//!   management
//! - `vs/platform/contextkey/common/contextkey.ts` - "When" clause evaluation
//!
//! ## TODO
//!
//! - [ ] Implement complete "when" clause expression parser and evaluator
//! - [ ] Add keybinding precedence scoring algorithm
//! - [ ] Support keybinding localization (different key labels per locale)
//! - [ ] Implement platform-specific modifier conversion (Cmd/Ctrl/Alt)
//! - [ ] Add conflict detection and warnings during registration
//! - [ ] Implement keybinding chords (e.g., "Ctrl+K Ctrl+C")
//! - [ ] Support custom keybinding schemes (vim, emacs, sublime)
//! - [ ] Add keybinding validation and syntax error reporting
//! - [ ] Implement keybinding telemetry for usage tracking
//! - [ ] Support keybinding migration across versions
//! - [ ] Add keybinding export/import functionality
//! - [ ] Implement keybinding search and discovery UI
//! - [ ] Support keybinding recording from actual key presses
//! - [ ] Add keybinding conflict resolution UI
//! - [ ] Implement keybinding per-profile (development, debugging, etc.)
//!
//! ## MODULE CONTENTS
//!
//! - [`KeybindingProvider`]: Main struct implementing the trait
//! - Keybinding collection and registration
//! - "When" clause parsing and evaluation
//! - Conflict resolution logic
//! - Platform-specific key mapping
//! - Command activation from keyboard events

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
use serde_json::{Value, json};
use tauri::Manager;

use super::{MountainEnvironment::MountainEnvironment, Utility};
use crate::{RunTime::ApplicationRunTime::ApplicationRunTime, dev_log};

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
